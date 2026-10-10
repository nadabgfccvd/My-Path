//! Windows backend: Raw Input on a hidden message-only window.
//!
//! * Mouse (usage 0x01/0x02) and keyboard (0x01/0x06) with `RIDEV_INPUTSINK`
//!   so we receive input even when another app has focus.
//! * Precision Touchpad HID reports (0x0D/0x05): contacts are parsed with the
//!   HID parser (`HidP_*`) and fed to `mp-gestures`.
//! * Mouse vs touchpad: decided per event from the raw-input device handle,
//!   VID/PID matching against the touchpad, and time correlation with touchpad
//!   contacts. The decision is recorded in `InputEvent::classification` so the
//!   Probe reports can tell us which heuristic is right (Phase 1, challenge 2).

use crate::CaptureError;
use crate::devname::parse_vid_pid;
use crate::keys::classify_vk;
use crossbeam_channel::Sender;
use mp_core::{
    Contact, DeviceInfo, DeviceKind, EventKind, InputEvent, MouseButton, WindowInfo, now_ms,
};
use mp_gestures::{Classifier, Config, Frame};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::size_of;
use std::thread::JoinHandle;
use windows::Win32::Devices::HumanInterfaceDevice::{
    HIDP_CAPS, HIDP_STATUS_SUCCESS, HIDP_VALUE_CAPS, HidP_GetCaps, HidP_GetUsageValue,
    HidP_GetUsages, HidP_GetValueCaps, HidP_Input, PHIDP_PREPARSED_DATA,
};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    GetCurrentThreadId, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    QueryFullProcessImageNameW,
};
use windows::Win32::UI::Input::{
    GetRawInputData, GetRawInputDeviceInfoW, GetRawInputDeviceList, HRAWINPUT, RAWINPUT,
    RAWINPUTDEVICE, RAWINPUTDEVICELIST, RAWINPUTHEADER, RID_DEVICE_INFO, RID_INPUT,
    RIDEV_INPUTSINK, RIDI_DEVICEINFO, RIDI_DEVICENAME, RIDI_PREPARSEDDATA, RIM_TYPEHID,
    RIM_TYPEKEYBOARD, RIM_TYPEMOUSE, RegisterRawInputDevices,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorPos, GetForegroundWindow,
    GetMessageW, GetWindowTextW, GetWindowThreadProcessId, HWND_MESSAGE, MSG, PostThreadMessageW,
    RegisterClassW, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_INPUT, WM_QUIT, WNDCLASSW,
};
use windows::core::{PWSTR, w};

// Raw-input mouse button flags (winuser.h).
const RI_MOUSE_LEFT_DOWN: u16 = 0x0001;
const RI_MOUSE_LEFT_UP: u16 = 0x0002;
const RI_MOUSE_RIGHT_DOWN: u16 = 0x0004;
const RI_MOUSE_RIGHT_UP: u16 = 0x0008;
const RI_MOUSE_MIDDLE_DOWN: u16 = 0x0010;
const RI_MOUSE_MIDDLE_UP: u16 = 0x0020;
const RI_MOUSE_X1_DOWN: u16 = 0x0040;
const RI_MOUSE_X1_UP: u16 = 0x0080;
const RI_MOUSE_X2_DOWN: u16 = 0x0100;
const RI_MOUSE_X2_UP: u16 = 0x0200;
const RI_MOUSE_WHEEL: u16 = 0x0400;
const RI_MOUSE_HWHEEL: u16 = 0x0800;
const RI_KEY_BREAK: u16 = 0x0001;

/// How long after touchpad contacts a handle-less mouse event is attributed to the touchpad.
const TOUCH_CORRELATION_MS: u64 = 250;

pub struct Handle {
    thread_id: u32,
    join: Option<JoinHandle<()>>,
}

impl Handle {
    pub fn stop(mut self) {
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

pub fn start(tx: Sender<InputEvent>) -> Result<Handle, CaptureError> {
    let (ready_tx, ready_rx) = crossbeam_channel::bounded::<Result<u32, String>>(1);
    let join = std::thread::Builder::new()
        .name("mp-input-rawinput".into())
        .spawn(move || run(tx, ready_tx))
        .map_err(|e| CaptureError::Other(e.to_string()))?;
    match ready_rx.recv() {
        Ok(Ok(thread_id)) => Ok(Handle {
            thread_id,
            join: Some(join),
        }),
        Ok(Err(e)) => Err(CaptureError::Other(e)),
        Err(_) => Err(CaptureError::Other("raw input thread exited".into())),
    }
}

fn run(tx: Sender<InputEvent>, ready: Sender<Result<u32, String>>) {
    unsafe {
        let hinstance = match GetModuleHandleW(None) {
            Ok(h) => h,
            Err(e) => {
                let _ = ready.send(Err(format!("GetModuleHandleW: {e}")));
                return;
            }
        };
        let class = w!("MyPathRawInputSink");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance.into(),
            lpszClassName: class,
            ..Default::default()
        };
        RegisterClassW(&wc);
        let hwnd = match CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class,
            w!("My Path input sink"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(hinstance.into()),
            None,
        ) {
            Ok(h) => h,
            Err(e) => {
                let _ = ready.send(Err(format!("CreateWindowExW: {e}")));
                return;
            }
        };

        let mut state = State::new(tx.clone());
        for info in list_devices() {
            state.remember(info);
        }
        STATE.with(|s| *s.borrow_mut() = Some(state));

        let dev = |page: u16, usage: u16| RAWINPUTDEVICE {
            usUsagePage: page,
            usUsage: usage,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };
        let all = [dev(0x01, 0x02), dev(0x01, 0x06), dev(0x0D, 0x05)];
        let size = size_of::<RAWINPUTDEVICE>() as u32;
        if RegisterRawInputDevices(&all, size).is_err() {
            // Touchpad registration can fail on machines without a precision touchpad.
            if let Err(e) = RegisterRawInputDevices(&all[..2], size) {
                let _ = ready.send(Err(format!("RegisterRawInputDevices: {e}")));
                return;
            }
            diag(
                &tx,
                "warn",
                "Registro do touchpad (HID 0x0D/0x05) falhou; só mouse e teclado.",
            );
        }
        let _ = ready.send(Ok(GetCurrentThreadId()));

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn diag(tx: &Sender<InputEvent>, level: &str, message: &str) {
    let _ = tx.send(InputEvent::new(
        "windows",
        DeviceKind::Unknown,
        EventKind::Diagnostic {
            level: level.into(),
            message: message.into(),
        },
    ));
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_INPUT {
        unsafe { handle_input(lp) };
    }
    unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
}

unsafe fn handle_input(lp: LPARAM) {
    let hraw = HRAWINPUT(lp.0 as *mut c_void);
    let header = size_of::<RAWINPUTHEADER>() as u32;
    let mut size = 0u32;
    unsafe { GetRawInputData(hraw, RID_INPUT, None, &mut size, header) };
    if size == 0 {
        return;
    }
    // u64 buffer guarantees alignment for RAWINPUT.
    let mut buf = vec![0u64; (size as usize).div_ceil(8)];
    let got = unsafe {
        GetRawInputData(
            hraw,
            RID_INPUT,
            Some(buf.as_mut_ptr() as *mut c_void),
            &mut size,
            header,
        )
    };
    if got == 0 || got == u32::MAX {
        return;
    }
    let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, size as usize) };
    let raw = unsafe { &*(buf.as_ptr() as *const RAWINPUT) };
    STATE.with(|s| {
        if let Some(st) = s.borrow_mut().as_mut() {
            unsafe { st.on_raw(raw, bytes) };
        }
    });
}

struct HidTouchpad {
    preparsed: Vec<u64>,
    finger_collections: Vec<u16>,
    classifier: Classifier,
    pending: Vec<Contact>,
    expected: usize,
    reported: usize,
    last_count: u8,
    last_button: bool,
}

impl HidTouchpad {
    fn new(preparsed: Vec<u64>) -> Option<Self> {
        let pp = PHIDP_PREPARSED_DATA(preparsed.as_ptr() as isize);
        let mut caps = HIDP_CAPS::default();
        if unsafe { HidP_GetCaps(pp, &mut caps) } != HIDP_STATUS_SUCCESS {
            return None;
        }
        let mut n = caps.NumberInputValueCaps;
        let mut vcaps = vec![HIDP_VALUE_CAPS::default(); n as usize];
        if unsafe { HidP_GetValueCaps(HidP_Input, vcaps.as_mut_ptr(), &mut n, pp) }
            != HIDP_STATUS_SUCCESS
        {
            return None;
        }
        let mut fingers = Vec::new();
        let (mut w, mut h) = (1000.0f32, 700.0f32);
        for c in &vcaps[..n as usize] {
            let usage = unsafe {
                if c.IsRange {
                    c.Anonymous.Range.UsageMin
                } else {
                    c.Anonymous.NotRange.Usage
                }
            };
            if c.UsagePage == 0x01 && usage == 0x30 {
                if !fingers.contains(&c.LinkCollection) {
                    fingers.push(c.LinkCollection);
                }
                w = (c.LogicalMax - c.LogicalMin).max(1) as f32;
            } else if c.UsagePage == 0x01 && usage == 0x31 {
                h = (c.LogicalMax - c.LogicalMin).max(1) as f32;
            }
        }
        if fingers.is_empty() {
            return None;
        }
        Some(Self {
            preparsed,
            finger_collections: fingers,
            classifier: Classifier::new(Config::with_pad_size(w, h)),
            pending: Vec::new(),
            expected: 0,
            reported: 0,
            last_count: 0,
            last_button: false,
        })
    }

    /// Parse one HID input report. Returns a complete frame when available.
    fn parse(&mut self, report: &mut [u8]) -> Option<Frame> {
        let pp = PHIDP_PREPARSED_DATA(self.preparsed.as_ptr() as isize);
        let value = |page: u16, lc: Option<u16>, usage: u16, report: &[u8]| -> Option<u32> {
            let mut v = 0u32;
            let st = unsafe { HidP_GetUsageValue(HidP_Input, page, lc, usage, &mut v, pp, report) };
            (st == HIDP_STATUS_SUCCESS).then_some(v)
        };
        let contact_count = value(0x0D, None, 0x54, report).unwrap_or(0) as usize;
        if contact_count > 0 {
            self.pending.clear();
            self.expected = contact_count;
            self.reported = 0;
        }
        for &lc in &self.finger_collections.clone() {
            let Some(x) = value(0x01, Some(lc), 0x30, report) else {
                continue;
            };
            let Some(y) = value(0x01, Some(lc), 0x31, report) else {
                continue;
            };
            let id = value(0x0D, Some(lc), 0x51, report).unwrap_or(lc as u32);
            let mut usages = [0u16; 32];
            let mut len = usages.len() as u32;
            let st = unsafe {
                HidP_GetUsages(
                    HidP_Input,
                    0x0D,
                    Some(lc),
                    usages.as_mut_ptr(),
                    &mut len,
                    pp,
                    report,
                )
            };
            let tip = st == HIDP_STATUS_SUCCESS && usages[..len as usize].contains(&0x42);
            if self.reported < self.expected.max(1) {
                self.reported += 1;
                if tip {
                    self.pending.push(Contact {
                        id,
                        x: x as i32,
                        y: y as i32,
                    });
                }
            }
        }
        let mut buttons = [0u16; 8];
        let mut blen = buttons.len() as u32;
        let st = unsafe {
            HidP_GetUsages(
                HidP_Input,
                0x09,
                None,
                buttons.as_mut_ptr(),
                &mut blen,
                pp,
                report,
            )
        };
        let button = st == HIDP_STATUS_SUCCESS && buttons[..blen as usize].contains(&1);

        if self.reported >= self.expected || (contact_count == 0 && self.expected == 0) {
            let frame = Frame {
                t_ms: now_ms(),
                contacts: std::mem::take(&mut self.pending),
                button,
            };
            self.expected = 0;
            self.reported = 0;
            return Some(frame);
        }
        None
    }
}

struct DevEntry {
    info: DeviceInfo,
    touchpad: Option<HidTouchpad>,
}

struct State {
    tx: Sender<InputEvent>,
    devices: HashMap<usize, DevEntry>,
    touchpad_ids: HashSet<(u16, u16)>,
    last_touch_ms: u64,
    pressed: HashSet<u16>,
}

impl State {
    fn new(tx: Sender<InputEvent>) -> Self {
        Self {
            tx,
            devices: HashMap::new(),
            touchpad_ids: HashSet::new(),
            last_touch_ms: 0,
            pressed: HashSet::new(),
        }
    }

    fn remember(&mut self, info: DeviceInfo) {
        if info.kind == DeviceKind::Touchpad
            && let (Some(v), Some(p)) = (info.vendor_id, info.product_id)
        {
            self.touchpad_ids.insert((v, p));
        }
        if let Some(key) = parse_handle_id(&info.id) {
            let touchpad = if info.kind == DeviceKind::Touchpad {
                preparsed(HANDLE(key as *mut c_void)).and_then(HidTouchpad::new)
            } else {
                None
            };
            self.devices.insert(key, DevEntry { info, touchpad });
        }
    }

    fn entry(&mut self, h: HANDLE) -> &mut DevEntry {
        let key = h.0 as usize;
        if !self.devices.contains_key(&key) {
            let info = describe(h);
            self.remember(info);
        }
        self.devices.entry(key).or_insert_with(|| DevEntry {
            info: describe(h),
            touchpad: None,
        })
    }

    fn send(&self, ev: InputEvent) {
        let _ = self.tx.send(ev);
    }

    unsafe fn on_raw(&mut self, raw: &RAWINPUT, bytes: &[u8]) {
        let kind = raw.header.dwType;
        let h = raw.header.hDevice;
        if kind == RIM_TYPEMOUSE.0 {
            let m = unsafe { raw.data.mouse.Anonymous.Anonymous };
            if m.usButtonFlags != 0 {
                self.on_mouse(h, m.usButtonFlags, m.usButtonData as i16);
            }
        } else if kind == RIM_TYPEKEYBOARD.0 {
            let k = unsafe { raw.data.keyboard };
            self.on_key(h, k.VKey, k.Flags);
        } else if kind == RIM_TYPEHID.0 {
            let hid = unsafe { raw.data.hid };
            let base = unsafe { std::ptr::addr_of!(raw.data.hid.bRawData) } as usize;
            let offset = base - bytes.as_ptr() as usize;
            let each = hid.dwSizeHid as usize;
            let total = each * hid.dwCount as usize;
            if each == 0 || offset + total > bytes.len() {
                return;
            }
            for i in 0..hid.dwCount as usize {
                let mut report = bytes[offset + i * each..offset + (i + 1) * each].to_vec();
                self.on_touch_report(h, &mut report);
            }
        }
    }

    fn on_touch_report(&mut self, h: HANDLE, report: &mut [u8]) {
        let id = format!("{:#x}", h.0 as usize);
        let entry = self.entry(h);
        let Some(tp) = entry.touchpad.as_mut() else {
            return;
        };
        let Some(frame) = tp.parse(report) else {
            return;
        };
        let count = frame.contacts.len() as u8;
        let mut out = Vec::new();
        if count != tp.last_count || frame.button != tp.last_button {
            out.push(EventKind::Contacts {
                count,
                contacts: frame.contacts.clone(),
                button: frame.button,
            });
            tp.last_count = count;
            tp.last_button = frame.button;
        }
        for g in tp.classifier.feed(&frame) {
            out.push(EventKind::Gesture(g));
        }
        if count > 0 || !out.is_empty() {
            self.last_touch_ms = now_ms();
        }
        for k in out {
            let mut ev = InputEvent::new(id.clone(), DeviceKind::Touchpad, k);
            ev.classification = Some("Precision Touchpad HID (0x0D/0x05) + mp-gestures".into());
            self.send(ev);
        }
    }

    fn classify_mouse(&mut self, h: HANDLE) -> (String, DeviceKind, String) {
        let since_touch = now_ms().saturating_sub(self.last_touch_ms);
        if h.0.is_null() {
            return if since_touch <= TOUCH_CORRELATION_MS {
                (
                    "0x0".into(),
                    DeviceKind::Touchpad,
                    format!("hDevice=0 + contatos no touchpad há {since_touch} ms"),
                )
            } else {
                (
                    "0x0".into(),
                    DeviceKind::Synthetic,
                    "hDevice=0 (entrada sintetizada)".into(),
                )
            };
        }
        let touchpad_ids = self.touchpad_ids.clone();
        let entry = self.entry(h);
        let info = entry.info.clone();
        if let (Some(v), Some(p)) = (info.vendor_id, info.product_id)
            && touchpad_ids.contains(&(v, p))
        {
            return (
                info.id,
                DeviceKind::Touchpad,
                "mesmo VID/PID do Precision Touchpad".into(),
            );
        }
        if since_touch <= TOUCH_CORRELATION_MS / 2 {
            return (
                info.id,
                DeviceKind::Touchpad,
                format!("correlação temporal com o touchpad ({since_touch} ms)"),
            );
        }
        (
            info.id,
            info.kind,
            format!("dispositivo de mouse: {}", short_name(&info.name)),
        )
    }

    fn on_mouse(&mut self, h: HANDLE, flags: u16, data: i16) {
        let (id, kind, class) = self.classify_mouse(h);
        let table = [
            (RI_MOUSE_LEFT_DOWN, MouseButton::Left, true),
            (RI_MOUSE_LEFT_UP, MouseButton::Left, false),
            (RI_MOUSE_RIGHT_DOWN, MouseButton::Right, true),
            (RI_MOUSE_RIGHT_UP, MouseButton::Right, false),
            (RI_MOUSE_MIDDLE_DOWN, MouseButton::Middle, true),
            (RI_MOUSE_MIDDLE_UP, MouseButton::Middle, false),
            (RI_MOUSE_X1_DOWN, MouseButton::X1, true),
            (RI_MOUSE_X1_UP, MouseButton::X1, false),
            (RI_MOUSE_X2_DOWN, MouseButton::X2, true),
            (RI_MOUSE_X2_UP, MouseButton::X2, false),
        ];
        let cursor = cursor_pos();
        let mut kinds = Vec::new();
        for (flag, button, down) in table {
            if flags & flag != 0 {
                kinds.push(if down {
                    EventKind::ButtonDown { button }
                } else {
                    EventKind::ButtonUp { button }
                });
            }
        }
        if flags & RI_MOUSE_WHEEL != 0 {
            kinds.push(EventKind::Wheel {
                delta: data as f32 / 120.0,
                horizontal: false,
            });
        }
        if flags & RI_MOUSE_HWHEEL != 0 {
            kinds.push(EventKind::Wheel {
                delta: data as f32 / 120.0,
                horizontal: true,
            });
        }
        for k in kinds {
            let window = matches!(k, EventKind::ButtonDown { .. }).then(foreground_window);
            let mut ev = InputEvent::new(id.clone(), kind, k);
            ev.cursor = cursor;
            ev.window = window.flatten();
            ev.classification = Some(class.clone());
            self.send(ev);
        }
    }

    fn on_key(&mut self, h: HANDLE, vk: u16, flags: u16) {
        if vk == 0xFF || vk == 0 {
            return;
        }
        let up = flags & RI_KEY_BREAK != 0;
        if up {
            self.pressed.remove(&vk);
        } else if !self.pressed.insert(vk) {
            return; // auto-repeat
        }
        let (class, name) = classify_vk(vk);
        let mut ev = InputEvent::new(
            format!("{:#x}", h.0 as usize),
            DeviceKind::Keyboard,
            EventKind::Key {
                class,
                down: !up,
                name: name.map(String::from),
            },
        );
        if !up {
            ev.window = foreground_window();
        }
        ev.classification = Some("Raw Input: teclado".into());
        self.send(ev);
    }
}

fn parse_handle_id(id: &str) -> Option<usize> {
    usize::from_str_radix(id.trim_start_matches("0x"), 16).ok()
}

fn short_name(name: &str) -> String {
    name.split('#').take(2).collect::<Vec<_>>().join("#")
}

fn cursor_pos() -> Option<(i32, i32)> {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p) }.ok().map(|_| (p.x, p.y))
}

fn foreground_window() -> Option<WindowInfo> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut buf = [0u16; 512];
        let n = GetWindowTextW(hwnd, &mut buf);
        let title = (n > 0).then(|| String::from_utf16_lossy(&buf[..n as usize]));
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let mut process = None;
        if let Ok(hp) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut name = [0u16; 1024];
            let mut len = name.len() as u32;
            if QueryFullProcessImageNameW(
                hp,
                PROCESS_NAME_WIN32,
                PWSTR(name.as_mut_ptr()),
                &mut len,
            )
            .is_ok()
            {
                let full = String::from_utf16_lossy(&name[..len as usize]);
                process = full.rsplit('\\').next().map(String::from);
            }
            let _ = CloseHandle(hp);
        }
        Some(WindowInfo { title, process })
    }
}

fn device_name(h: HANDLE) -> Option<String> {
    unsafe {
        let mut size = 0u32;
        GetRawInputDeviceInfoW(Some(h), RIDI_DEVICENAME, None, &mut size);
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u16; size as usize + 1];
        let r = GetRawInputDeviceInfoW(
            Some(h),
            RIDI_DEVICENAME,
            Some(buf.as_mut_ptr() as *mut c_void),
            &mut size,
        );
        if r == u32::MAX {
            return None;
        }
        let s = String::from_utf16_lossy(&buf);
        Some(s.trim_end_matches('\0').to_string())
    }
}

fn preparsed(h: HANDLE) -> Option<Vec<u64>> {
    unsafe {
        let mut size = 0u32;
        GetRawInputDeviceInfoW(Some(h), RIDI_PREPARSEDDATA, None, &mut size);
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u64; (size as usize).div_ceil(8)];
        let r = GetRawInputDeviceInfoW(
            Some(h),
            RIDI_PREPARSEDDATA,
            Some(buf.as_mut_ptr() as *mut c_void),
            &mut size,
        );
        (r != u32::MAX && r > 0).then_some(buf)
    }
}

fn describe(h: HANDLE) -> DeviceInfo {
    let name = device_name(h).unwrap_or_else(|| "?".into());
    let (mut vid, mut pid) = parse_vid_pid(&name);
    let mut kind = DeviceKind::Unknown;
    let mut notes = Vec::new();
    unsafe {
        let mut info = RID_DEVICE_INFO {
            cbSize: size_of::<RID_DEVICE_INFO>() as u32,
            ..Default::default()
        };
        let mut size = info.cbSize;
        let r = GetRawInputDeviceInfoW(
            Some(h),
            RIDI_DEVICEINFO,
            Some(&mut info as *mut _ as *mut c_void),
            &mut size,
        );
        if r != u32::MAX && r > 0 {
            if info.dwType == RIM_TYPEMOUSE {
                kind = DeviceKind::Mouse;
            } else if info.dwType == RIM_TYPEKEYBOARD {
                kind = DeviceKind::Keyboard;
            } else if info.dwType == RIM_TYPEHID {
                let hid = info.Anonymous.hid;
                vid = vid.or(Some(hid.dwVendorId as u16));
                pid = pid.or(Some(hid.dwProductId as u16));
                notes.push(format!(
                    "HID page {:#04x} usage {:#04x}",
                    hid.usUsagePage, hid.usUsage
                ));
                kind = match (hid.usUsagePage, hid.usUsage) {
                    (0x0D, 0x05) => DeviceKind::Touchpad,
                    (0x0D, 0x04) => DeviceKind::Touchscreen,
                    (0x0D, 0x02) => DeviceKind::Pen,
                    _ => DeviceKind::Unknown,
                };
            }
        }
    }
    let lname = name.to_ascii_lowercase();
    if kind == DeviceKind::Mouse
        && ["syn", "etd", "elan", "alps", "synaptics"]
            .iter()
            .any(|k| lname.contains(k))
    {
        notes.push("nome sugere touchpad legado (Synaptics/ELAN/ALPS)".into());
    }
    DeviceInfo {
        id: format!("{:#x}", h.0 as usize),
        kind,
        name,
        vendor_id: vid,
        product_id: pid,
        notes,
    }
}

pub fn list_devices() -> Vec<DeviceInfo> {
    unsafe {
        let size = size_of::<RAWINPUTDEVICELIST>() as u32;
        let mut n = 0u32;
        GetRawInputDeviceList(None, &mut n, size);
        if n == 0 {
            return Vec::new();
        }
        let mut list = vec![RAWINPUTDEVICELIST::default(); n as usize];
        let got = GetRawInputDeviceList(Some(list.as_mut_ptr()), &mut n, size);
        if got == u32::MAX {
            return Vec::new();
        }
        list.truncate(got as usize);
        list.iter()
            .map(|d| describe(d.hDevice))
            .filter(|i| i.kind != DeviceKind::Unknown)
            .collect()
    }
}
