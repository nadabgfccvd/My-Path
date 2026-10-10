//! # mp-core
//!
//! Shared data model for My Path. Every other crate speaks these types:
//! devices, raw input events, recognised gestures and the environment report
//! produced by the Probe.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Milliseconds since UNIX epoch. Used for every timestamp in My Path.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// What kind of physical device produced an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Mouse,
    Touchpad,
    Keyboard,
    Touchscreen,
    Pen,
    /// Input synthesised by the OS (e.g. Windows precision-touchpad taps that
    /// reach raw input without a device handle). Phase 1 investigates these.
    Synthetic,
    Unknown,
}

/// A discovered input device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Stable id within one session (e.g. raw-input handle or /dev/input path).
    pub id: String,
    pub kind: DeviceKind,
    /// Human readable name reported by the OS/driver.
    pub name: String,
    /// USB/HID vendor id, if known.
    pub vendor_id: Option<u16>,
    /// USB/HID product id, if known.
    pub product_id: Option<u16>,
    /// Free-form notes collected during detection (capabilities, heuristics…).
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    /// Side button "back".
    X1,
    /// Side button "forward".
    X2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinchKind {
    /// Fingers move apart -> zoom in.
    Out,
    /// Fingers move together -> zoom out.
    In,
}

/// Privacy-preserving key category. The Probe never stores which letter was typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyClass {
    Letter,
    Digit,
    Modifier,
    Navigation,
    Function,
    Editing,
    Other,
}

/// One touch contact on a touchpad / touchscreen, in device units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    pub id: u32,
    pub x: i32,
    pub y: i32,
}

/// Recognised touchpad gesture (output of `mp-gestures`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "gesture", rename_all = "snake_case")]
pub enum Gesture {
    Tap { fingers: u8 },
    DoubleTap { fingers: u8 },
    PhysicalClick { fingers: u8 },
    Scroll { fingers: u8, direction: Direction },
    Swipe { fingers: u8, direction: Direction },
    Pinch { fingers: u8, kind: PinchKind },
    Hold { fingers: u8 },
}

/// Information about the window that had focus when an event happened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WindowInfo {
    pub title: Option<String>,
    pub process: Option<String>,
}

/// Every event the capture layer can report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    ButtonDown {
        button: MouseButton,
    },
    ButtonUp {
        button: MouseButton,
    },
    /// Wheel movement. `delta` > 0 means up (vertical) or right (horizontal),
    /// expressed in "notches" (120 units on Windows = 1.0).
    Wheel {
        delta: f32,
        horizontal: bool,
    },
    Key {
        class: KeyClass,
        down: bool,
        name: Option<String>,
    },
    /// Number of fingers currently touching a touchpad changed.
    Contacts {
        count: u8,
        contacts: Vec<Contact>,
        button: bool,
    },
    Gesture(Gesture),
    /// Diagnostic message from a backend (permission problems, errors…).
    Diagnostic {
        level: String,
        message: String,
    },
}

/// A timestamped event coming from a device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputEvent {
    pub t_ms: u64,
    pub device_id: String,
    pub device_kind: DeviceKind,
    pub kind: EventKind,
    /// Cursor position in screen pixels, when the platform can tell.
    pub cursor: Option<(i32, i32)>,
    /// Foreground window, when the platform can tell.
    pub window: Option<WindowInfo>,
    /// How the device kind was decided (for Phase 1 analysis).
    pub classification: Option<String>,
}

impl InputEvent {
    pub fn new(device_id: impl Into<String>, device_kind: DeviceKind, kind: EventKind) -> Self {
        Self {
            t_ms: now_ms(),
            device_id: device_id.into(),
            device_kind,
            kind,
            cursor: None,
            window: None,
            classification: None,
        }
    }
}

/// Snapshot of the machine/session, shown by the Probe and saved in reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Environment {
    pub os_family: String,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub arch: String,
    pub hostname_hash: Option<String>,
    /// Linux: "x11" | "wayland" | "tty"… ; Windows: None.
    pub session_type: Option<String>,
    pub desktop: Option<String>,
    pub app_version: String,
    pub notes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serializes_with_tags() {
        let ev = InputEvent::new(
            "dev0",
            DeviceKind::Touchpad,
            EventKind::Gesture(Gesture::Swipe {
                fingers: 3,
                direction: Direction::Up,
            }),
        );
        let json = serde_json::to_string(&ev).unwrap();
        assert!(json.contains("\"type\":\"gesture\""));
        assert!(json.contains("\"gesture\":\"swipe\""));
        assert!(json.contains("\"device_kind\":\"touchpad\""));
        let back: InputEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ev);
    }
}
