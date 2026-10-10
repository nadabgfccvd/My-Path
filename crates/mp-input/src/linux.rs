//! Linux backend: reads `/dev/input/event*` directly with evdev.
//!
//! Works the same on X11 and Wayland because it sits below the display
//! server. Requires read permission on the device nodes (group `input` or a
//! udev `uaccess` rule). Cursor position and focused window are NOT available
//! here; they come from other backends in later steps (see LIMITACOES.md).

use crate::CaptureError;
use crate::keys::classify_evdev;
use crossbeam_channel::Sender;
use evdev::raw_stream::RawDevice;
use evdev::{AbsoluteAxisCode, EventType, KeyCode, PropType, RelativeAxisCode};
use mp_core::{Contact, DeviceInfo, DeviceKind, EventKind, InputEvent, MouseButton};
use mp_gestures::{Classifier, Config, Frame};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

pub struct Handle {
    stop: Arc<AtomicBool>,
    _threads: Vec<JoinHandle<()>>,
}

impl Handle {
    pub fn stop(self) {
        // Reader threads block on the device; they exit on their next event.
        self.stop.store(true, Ordering::SeqCst);
    }
}

fn event_paths() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir("/dev/input")
        .map(|d| {
            d.flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .map(|n| n.to_string_lossy().starts_with("event"))
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn classify(dev: &RawDevice) -> (DeviceKind, Vec<String>) {
    let keys = dev.supported_keys();
    let abs = dev.supported_absolute_axes();
    let rel = dev.supported_relative_axes();
    let props = dev.properties();
    let has_key = |k: KeyCode| keys.map(|s| s.contains(k)).unwrap_or(false);
    let has_abs = |a: AbsoluteAxisCode| abs.map(|s| s.contains(a)).unwrap_or(false);
    let has_rel = |r: RelativeAxisCode| rel.map(|s| s.contains(r)).unwrap_or(false);
    let mut notes = Vec::new();
    let mt = has_abs(AbsoluteAxisCode::ABS_MT_POSITION_X);
    if mt {
        notes.push("multitouch (ABS_MT_POSITION_X)".into());
    }
    if props.contains(PropType::BUTTONPAD) {
        notes.push("clickpad (INPUT_PROP_BUTTONPAD)".into());
    }
    let kind = if mt && props.contains(PropType::DIRECT) {
        DeviceKind::Touchscreen
    } else if has_key(KeyCode::BTN_TOOL_PEN) {
        DeviceKind::Pen
    } else if (mt || has_abs(AbsoluteAxisCode::ABS_X)) && has_key(KeyCode::BTN_TOOL_FINGER) {
        DeviceKind::Touchpad
    } else if has_rel(RelativeAxisCode::REL_X) && has_key(KeyCode::BTN_LEFT) {
        DeviceKind::Mouse
    } else if has_key(KeyCode::KEY_A) && has_key(KeyCode::KEY_Z) && has_key(KeyCode::KEY_ENTER) {
        DeviceKind::Keyboard
    } else {
        DeviceKind::Unknown
    };
    (kind, notes)
}

fn info_for(path: &Path, dev: &RawDevice) -> DeviceInfo {
    let (kind, notes) = classify(dev);
    let id = dev.input_id();
    DeviceInfo {
        id: path.display().to_string(),
        kind,
        name: dev.name().unwrap_or("?").to_string(),
        vendor_id: Some(id.vendor()),
        product_id: Some(id.product()),
        notes,
    }
}

pub fn list_devices() -> Vec<DeviceInfo> {
    event_paths()
        .into_iter()
        .map(|p| match RawDevice::open(&p) {
            Ok(dev) => info_for(&p, &dev),
            Err(e) => DeviceInfo {
                id: p.display().to_string(),
                kind: DeviceKind::Unknown,
                name: "(sem acesso)".into(),
                vendor_id: None,
                product_id: None,
                notes: vec![format!("erro ao abrir: {e}")],
            },
        })
        .collect()
}

pub fn start(tx: Sender<InputEvent>) -> Result<Handle, CaptureError> {
    let stop = Arc::new(AtomicBool::new(false));
    let mut threads = Vec::new();
    let mut denied = 0;
    for path in event_paths() {
        match RawDevice::open(&path) {
            Ok(dev) => {
                let info = info_for(&path, &dev);
                if matches!(info.kind, DeviceKind::Unknown) {
                    continue;
                }
                let tx = tx.clone();
                let stop = stop.clone();
                let t = std::thread::Builder::new()
                    .name(format!("mp-evdev-{}", info.id))
                    .spawn(move || reader(dev, info, tx, stop))
                    .map_err(|e| CaptureError::Other(e.to_string()))?;
                threads.push(t);
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => denied += 1,
            Err(_) => {}
        }
    }
    if denied > 0 {
        let _ = tx.send(InputEvent::new(
            "linux",
            DeviceKind::Unknown,
            EventKind::Diagnostic {
                level: "warn".into(),
                message: format!(
                    "{denied} dispositivo(s) em /dev/input sem permissão de leitura. \
                     Adicione seu usuário ao grupo 'input' (sudo usermod -aG input $USER) \
                     e entre de novo na sessão."
                ),
            },
        ));
    }
    if threads.is_empty() {
        return Err(CaptureError::Other(
            "nenhum dispositivo de entrada legível em /dev/input (permissão?)".into(),
        ));
    }
    Ok(Handle {
        stop,
        _threads: threads,
    })
}

/// Per-device multitouch state for touchpads.
struct TouchState {
    slots: Vec<Option<(i32, i32, i32)>>, // (tracking id, x, y)
    slot: usize,
    tool_fingers: u8,
    button: bool,
    classifier: Classifier,
    last_count: u8,
    last_button: bool,
}

impl TouchState {
    fn new(dev: &RawDevice) -> Self {
        let mut w = 1000.0;
        let mut h = 700.0;
        let mut nslots = 10usize;
        if let Ok(infos) = dev.get_absinfo() {
            for (code, info) in infos {
                let span = (info.maximum() - info.minimum()).max(1) as f32;
                if code == AbsoluteAxisCode::ABS_MT_POSITION_X {
                    w = span;
                } else if code == AbsoluteAxisCode::ABS_MT_POSITION_Y {
                    h = span;
                } else if code == AbsoluteAxisCode::ABS_MT_SLOT {
                    nslots = (info.maximum().max(0) as usize + 1).clamp(1, 16);
                }
            }
        }
        Self {
            slots: vec![None; nslots],
            slot: 0,
            tool_fingers: 0,
            button: false,
            classifier: Classifier::new(Config::with_pad_size(w, h)),
            last_count: 0,
            last_button: false,
        }
    }

    fn frame(&self, t_ms: u64) -> Frame {
        let mut contacts: Vec<Contact> = self
            .slots
            .iter()
            .flatten()
            .map(|(id, x, y)| Contact {
                id: *id as u32,
                x: *x,
                y: *y,
            })
            .collect();
        // Older touchpads track fewer slots than fingers (BTN_TOOL_TRIPLETAP with
        // 2 slots). Add synthetic contacts so finger counting stays correct.
        let need = self.tool_fingers as usize;
        if need > contacts.len()
            && let Some(first) = contacts.first().copied()
        {
            for k in contacts.len()..need {
                contacts.push(Contact {
                    id: 1000 + k as u32,
                    x: first.x + 60 * k as i32,
                    y: first.y,
                });
            }
        }
        if self.tool_fingers == 0 {
            contacts.clear();
        }
        Frame {
            t_ms,
            contacts,
            button: self.button,
        }
    }
}

fn reader(mut dev: RawDevice, info: DeviceInfo, tx: Sender<InputEvent>, stop: Arc<AtomicBool>) {
    let mut touch = matches!(info.kind, DeviceKind::Touchpad).then(|| TouchState::new(&dev));
    let kind = info.kind;
    let id = info.id.clone();
    let send = |k: EventKind, class: &str| {
        let mut ev = InputEvent::new(id.clone(), kind, k);
        ev.classification = Some(class.to_string());
        let _ = tx.send(ev);
    };
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        let events: Vec<evdev::InputEvent> = match dev.fetch_events() {
            Ok(it) => it.collect(),
            Err(e) => {
                send(
                    EventKind::Diagnostic {
                        level: "error".into(),
                        message: format!("{id}: {e}"),
                    },
                    "evdev",
                );
                return;
            }
        };
        for ev in events {
            let code = ev.code();
            let value = ev.value();
            match ev.event_type() {
                EventType::KEY => {
                    let key = KeyCode::new(code);
                    let button = match key {
                        KeyCode::BTN_LEFT => Some(MouseButton::Left),
                        KeyCode::BTN_RIGHT => Some(MouseButton::Right),
                        KeyCode::BTN_MIDDLE => Some(MouseButton::Middle),
                        KeyCode::BTN_SIDE | KeyCode::BTN_BACK => Some(MouseButton::X1),
                        KeyCode::BTN_EXTRA | KeyCode::BTN_FORWARD => Some(MouseButton::X2),
                        _ => None,
                    };
                    if let Some(t) = touch.as_mut() {
                        let fingers = match key {
                            KeyCode::BTN_TOOL_FINGER => Some(1),
                            KeyCode::BTN_TOOL_DOUBLETAP => Some(2),
                            KeyCode::BTN_TOOL_TRIPLETAP => Some(3),
                            KeyCode::BTN_TOOL_QUADTAP => Some(4),
                            KeyCode::BTN_TOOL_QUINTTAP => Some(5),
                            _ => None,
                        };
                        if let Some(n) = fingers {
                            if value == 1 {
                                t.tool_fingers = n;
                            } else if t.tool_fingers == n {
                                t.tool_fingers = 0;
                            }
                            continue;
                        }
                        if key == KeyCode::BTN_LEFT {
                            t.button = value != 0;
                        }
                    }
                    if let Some(b) = button {
                        if value == 2 {
                            continue;
                        }
                        let k = if value == 1 {
                            EventKind::ButtonDown { button: b }
                        } else {
                            EventKind::ButtonUp { button: b }
                        };
                        let class = if touch.is_some() {
                            "evdev: clickpad pressionado (lado/dedos decididos pelo compositor)"
                        } else {
                            "evdev: dispositivo de mouse"
                        };
                        send(k, class);
                    } else if matches!(kind, DeviceKind::Keyboard) && value != 2 {
                        let (class, name) = classify_evdev(code);
                        send(
                            EventKind::Key {
                                class,
                                down: value == 1,
                                name: name.map(String::from),
                            },
                            "evdev: teclado",
                        );
                    }
                }
                EventType::RELATIVE => {
                    let axis = RelativeAxisCode(code);
                    if axis == RelativeAxisCode::REL_WHEEL {
                        send(
                            EventKind::Wheel {
                                delta: value as f32,
                                horizontal: false,
                            },
                            "evdev: roda do mouse",
                        );
                    } else if axis == RelativeAxisCode::REL_HWHEEL {
                        send(
                            EventKind::Wheel {
                                delta: value as f32,
                                horizontal: true,
                            },
                            "evdev: roda horizontal",
                        );
                    }
                }
                EventType::ABSOLUTE => {
                    if let Some(t) = touch.as_mut() {
                        let axis = AbsoluteAxisCode(code);
                        if axis == AbsoluteAxisCode::ABS_MT_SLOT {
                            t.slot = (value.max(0) as usize).min(t.slots.len() - 1);
                        } else if axis == AbsoluteAxisCode::ABS_MT_TRACKING_ID {
                            t.slots[t.slot] = if value < 0 { None } else { Some((value, 0, 0)) };
                        } else if axis == AbsoluteAxisCode::ABS_MT_POSITION_X {
                            if let Some(s) = t.slots[t.slot].as_mut() {
                                s.1 = value;
                            }
                        } else if axis == AbsoluteAxisCode::ABS_MT_POSITION_Y
                            && let Some(s) = t.slots[t.slot].as_mut()
                        {
                            s.2 = value;
                        }
                    }
                }
                EventType::SYNCHRONIZATION => {
                    if let Some(t) = touch.as_mut() {
                        let frame = t.frame(mp_core::now_ms());
                        let count = frame.contacts.len() as u8;
                        if count != t.last_count || frame.button != t.last_button {
                            send(
                                EventKind::Contacts {
                                    count,
                                    contacts: frame.contacts.clone(),
                                    button: frame.button,
                                },
                                "evdev: contatos do touchpad",
                            );
                            t.last_count = count;
                            t.last_button = frame.button;
                        }
                        for g in t.classifier.feed(&frame) {
                            send(EventKind::Gesture(g), "mp-gestures (evdev)");
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
