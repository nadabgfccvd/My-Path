//! Privacy-preserving key classification shared by all backends.
//! Letters and digits are never named, only their category is kept.

#![allow(dead_code)]

use mp_core::KeyClass;

/// Windows virtual-key code -> (class, optional readable name).
pub fn classify_vk(vk: u16) -> (KeyClass, Option<&'static str>) {
    match vk {
        0x41..=0x5A => (KeyClass::Letter, None),
        0x30..=0x39 | 0x60..=0x69 => (KeyClass::Digit, None),
        0x10 => (KeyClass::Modifier, Some("Shift")),
        0x11 => (KeyClass::Modifier, Some("Ctrl")),
        0x12 => (KeyClass::Modifier, Some("Alt")),
        0xA0 => (KeyClass::Modifier, Some("Shift (esq.)")),
        0xA1 => (KeyClass::Modifier, Some("Shift (dir.)")),
        0xA2 => (KeyClass::Modifier, Some("Ctrl (esq.)")),
        0xA3 => (KeyClass::Modifier, Some("Ctrl (dir.)")),
        0xA4 => (KeyClass::Modifier, Some("Alt (esq.)")),
        0xA5 => (KeyClass::Modifier, Some("AltGr")),
        0x5B => (KeyClass::Modifier, Some("Win (esq.)")),
        0x5C => (KeyClass::Modifier, Some("Win (dir.)")),
        0x25 => (KeyClass::Navigation, Some("Seta esquerda")),
        0x26 => (KeyClass::Navigation, Some("Seta cima")),
        0x27 => (KeyClass::Navigation, Some("Seta direita")),
        0x28 => (KeyClass::Navigation, Some("Seta baixo")),
        0x21 => (KeyClass::Navigation, Some("Page Up")),
        0x22 => (KeyClass::Navigation, Some("Page Down")),
        0x23 => (KeyClass::Navigation, Some("End")),
        0x24 => (KeyClass::Navigation, Some("Home")),
        0x70..=0x87 => (KeyClass::Function, Some(FKEYS[(vk - 0x70) as usize])),
        0x08 => (KeyClass::Editing, Some("Backspace")),
        0x09 => (KeyClass::Editing, Some("Tab")),
        0x0D => (KeyClass::Editing, Some("Enter")),
        0x1B => (KeyClass::Editing, Some("Esc")),
        0x20 => (KeyClass::Editing, Some("Espaço")),
        0x2D => (KeyClass::Editing, Some("Insert")),
        0x2E => (KeyClass::Editing, Some("Delete")),
        0x14 => (KeyClass::Modifier, Some("Caps Lock")),
        _ => (KeyClass::Other, None),
    }
}

/// Linux evdev key code -> (class, optional readable name).
pub fn classify_evdev(code: u16) -> (KeyClass, Option<&'static str>) {
    match code {
        16..=25 | 30..=38 | 44..=50 => (KeyClass::Letter, None),
        2..=11 | 71..=73 | 75..=77 | 79..=82 => (KeyClass::Digit, None),
        29 => (KeyClass::Modifier, Some("Ctrl (esq.)")),
        97 => (KeyClass::Modifier, Some("Ctrl (dir.)")),
        42 => (KeyClass::Modifier, Some("Shift (esq.)")),
        54 => (KeyClass::Modifier, Some("Shift (dir.)")),
        56 => (KeyClass::Modifier, Some("Alt (esq.)")),
        100 => (KeyClass::Modifier, Some("AltGr")),
        125 => (KeyClass::Modifier, Some("Super (esq.)")),
        126 => (KeyClass::Modifier, Some("Super (dir.)")),
        58 => (KeyClass::Modifier, Some("Caps Lock")),
        103 => (KeyClass::Navigation, Some("Seta cima")),
        105 => (KeyClass::Navigation, Some("Seta esquerda")),
        106 => (KeyClass::Navigation, Some("Seta direita")),
        108 => (KeyClass::Navigation, Some("Seta baixo")),
        102 => (KeyClass::Navigation, Some("Home")),
        107 => (KeyClass::Navigation, Some("End")),
        104 => (KeyClass::Navigation, Some("Page Up")),
        109 => (KeyClass::Navigation, Some("Page Down")),
        59..=68 => (KeyClass::Function, Some(FKEYS[(code - 59) as usize])),
        87 => (KeyClass::Function, Some("F11")),
        88 => (KeyClass::Function, Some("F12")),
        14 => (KeyClass::Editing, Some("Backspace")),
        15 => (KeyClass::Editing, Some("Tab")),
        28 | 96 => (KeyClass::Editing, Some("Enter")),
        1 => (KeyClass::Editing, Some("Esc")),
        57 => (KeyClass::Editing, Some("Espaço")),
        110 => (KeyClass::Editing, Some("Insert")),
        111 => (KeyClass::Editing, Some("Delete")),
        _ => (KeyClass::Other, None),
    }
}

const FKEYS: [&str; 24] = [
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13", "F14", "F15",
    "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24",
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn letters_are_never_named() {
        for vk in 0x41..=0x5Au16 {
            assert_eq!(classify_vk(vk), (KeyClass::Letter, None));
        }
        assert_eq!(classify_evdev(30), (KeyClass::Letter, None)); // KEY_A
        assert_eq!(classify_evdev(29).0, KeyClass::Modifier);
        assert_eq!(classify_vk(0x74), (KeyClass::Function, Some("F5")));
        assert_eq!(classify_evdev(63), (KeyClass::Function, Some("F5")));
    }
}
