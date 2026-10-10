//! # mp-input
//!
//! Global input capture for My Path.
//!
//! * **Windows**: Raw Input (`RegisterRawInputDevices` with `RIDEV_INPUTSINK`)
//!   for mouse, keyboard and Precision Touchpad HID reports (Usage Page 0x0D,
//!   Usage 0x05). Touchpad contacts are parsed with `HidP_*` and classified by
//!   `mp-gestures`.
//! * **Linux** (X11 *and* Wayland): direct `evdev` reading of
//!   `/dev/input/event*` (needs read permission, see `docs/LIMITACOES.md`).
//!
//! Events are pushed into a `crossbeam_channel::Sender<InputEvent>`.

mod devname;
pub mod env;
mod keys;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(windows)]
mod win;

pub use crossbeam_channel::{Receiver, Sender, unbounded};
use mp_core::{DeviceInfo, InputEvent};

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("input capture is not supported on this platform yet")]
    Unsupported,
    #[error("{0}")]
    Other(String),
}

/// Running capture. Dropping it does not stop capture; call [`Capture::stop`].
pub struct Capture {
    #[cfg(windows)]
    inner: win::Handle,
    #[cfg(target_os = "linux")]
    inner: linux::Handle,
}

impl Capture {
    pub fn stop(self) {
        #[cfg(any(windows, target_os = "linux"))]
        self.inner.stop();
    }
}

/// Start capturing global input. Events are sent to `tx`.
pub fn start(tx: Sender<InputEvent>) -> Result<Capture, CaptureError> {
    #[cfg(windows)]
    {
        return Ok(Capture {
            inner: win::start(tx)?,
        });
    }
    #[cfg(target_os = "linux")]
    {
        return Ok(Capture {
            inner: linux::start(tx)?,
        });
    }
    #[allow(unreachable_code)]
    {
        let _ = tx;
        Err(CaptureError::Unsupported)
    }
}

/// List input devices currently visible to the process.
pub fn list_devices() -> Vec<DeviceInfo> {
    #[cfg(windows)]
    {
        return win::list_devices();
    }
    #[cfg(target_os = "linux")]
    {
        return linux::list_devices();
    }
    #[allow(unreachable_code)]
    Vec::new()
}
