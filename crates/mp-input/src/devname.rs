//! Helpers to parse Windows device interface paths (platform-independent so they are tested everywhere).

/// Extract VID/PID from a device interface path. Handles both
/// `HID#VID_06CB&PID_CE26&Col01` (USB/I2C) and `_VID&0002046d_PID&b023` (Bluetooth).
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn parse_vid_pid(name: &str) -> (Option<u16>, Option<u16>) {
    let up = name.to_ascii_uppercase();
    let grab = |tag: &str| -> Option<u16> {
        let i = up.find(tag)? + tag.len();
        let rest = up[i..].trim_start_matches(['_', '&']);
        let hex: String = rest
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .take(8)
            .collect();
        if hex.is_empty() {
            return None;
        }
        u32::from_str_radix(&hex, 16)
            .ok()
            .map(|v| (v & 0xFFFF) as u16)
    };
    (grab("VID"), grab("PID"))
}

#[cfg(test)]
mod tests {
    use super::parse_vid_pid;
    #[test]
    fn vid_pid_usb_and_bluetooth() {
        assert_eq!(
            parse_vid_pid(r"\\?\HID#VID_06CB&PID_CE26&Col01#5&1234#{guid}"),
            (Some(0x06CB), Some(0xCE26))
        );
        assert_eq!(
            parse_vid_pid(r"\\?\HID#{00001124-0000}_VID&0002046d_PID&b023&Col01#8&1"),
            (Some(0x046D), Some(0xB023))
        );
    }
}
