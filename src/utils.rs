/// Checks if the given key state corresponds to a system shortcut.
/// It detects if Ctrl, Alt, or Super (Windows/Command) keys are pressed.
pub fn is_shortcut(state: u32) -> bool {
    const CTRL: u32 = 1 << 2;
    const ALT: u32 = 1 << 3;
    const SUPER: u32 = 1 << 26;
    (state & (CTRL | ALT | SUPER)) != 0
}

/// Converts an X11 keyval into a printable character if possible.
/// Currently only supports standard ASCII graphic characters and Space.
pub fn keyval_to_char(keyval: u32) -> Option<char> {
    if keyval <= 0x7f {
        let c = keyval as u8 as char;
        if c.is_ascii_graphic() || c == ' ' {
            return Some(c);
        }
    }
    None
}
