//! Utility Functions for XKey Input Method
//!
//! This module provides helper functions for:
//! - Key state analysis (detecting shortcuts and modifiers)
//! - Keyval to character conversion

/// Checks if the given key state corresponds to a system shortcut.
///
/// System shortcuts should not be processed by the input method and should
/// be passed through directly to the application/window manager. This includes
/// keyboard shortcuts like:
/// - Ctrl+C, Ctrl+V, Ctrl+Z (common editing shortcuts)
/// - Alt+Tab, Alt+F4 (window management)
/// - Super+... (system shortcuts on Linux/Windows)
///
/// # Arguments
/// * `state` - The X11/IBus key state bitmask containing modifier information
///
/// # Returns
/// * `true` if any of Ctrl, Alt, or Super keys are pressed
/// * `false` otherwise
///
/// # X11 Modifier Masks
/// - Bit 2 (0x04): Control key
/// - Bit 3 (0x08): Alt key (Mod1)
/// - Bit 26 (0x04000000): Super key (Mod4)
pub fn is_shortcut(state: u32) -> bool {
    const CTRL: u32 = 1 << 2; // Control key modifier mask
    const ALT: u32 = 1 << 3; // Alt key modifier mask
    const SUPER: u32 = 1 << 26; // Super (Windows/Command) key modifier mask
    (state & (CTRL | ALT | SUPER)) != 0
}

/// Converts an X11 keyval into a printable character if possible.
///
/// This function handles the conversion from X11 keysyms to actual characters
/// that can be typed. It only supports standard ASCII graphic characters and
/// the space character.
///
/// # Arguments
/// * `keyval` - X11 keysym value
///
/// # Returns
/// * `Some(char)` if the keyval corresponds to a printable ASCII character
/// * `None` for special keys (Shift, Ctrl, arrows, function keys, etc.)
///
/// # Keysym Ranges
/// X11 keysyms for printable ASCII characters are in the range 0x20-0x7e:
/// - 0x20: Space
/// - 0x21-0x2f: Punctuation (!, ", #, $, etc.)
/// - 0x30-0x39: Digits (0-9)
/// - 0x41-0x5a: Uppercase letters (A-Z)
/// - 0x61-0x7a: Lowercase letters (a-z)
///
/// Special keys use higher values in the 0xff00+ range:
/// - 0xff08: Backspace
/// - 0xff09: Tab
/// - 0xff0d: Return/Enter
/// - 0xff1b: Escape
/// - 0xffe1-0xffe4: Shift, Caps Lock, Ctrl
pub fn keyval_to_char(keyval: u32) -> Option<char> {
    // Only handle standard ASCII range (0x00-0x7f)
    if keyval <= 0x7f {
        let c = keyval as u8 as char;
        // Return character only if it's a printable graphic or space
        // is_ascii_graphic() matches 0x21-0x7e (excludes control chars and space)
        if c.is_ascii_graphic() || c == ' ' {
            return Some(c);
        }
    }
    None
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_shortcut() {
        // No modifiers - should pass through
        assert!(!is_shortcut(0));

        // Ctrl pressed - is a shortcut
        assert!(is_shortcut(1 << 2));

        // Alt pressed - is a shortcut
        assert!(is_shortcut(1 << 3));

        // Super pressed - is a shortcut
        assert!(is_shortcut(1 << 26));

        // Shift only - NOT a shortcut (should allow Shift+letter for uppercase)
        assert!(!is_shortcut(1 << 0));
    }

    #[test]
    fn test_keyval_to_char() {
        // Lowercase letters
        assert_eq!(keyval_to_char(0x61), Some('a'));
        assert_eq!(keyval_to_char(0x7a), Some('z'));

        // Uppercase letters
        assert_eq!(keyval_to_char(0x41), Some('A'));
        assert_eq!(keyval_to_char(0x5a), Some('Z'));

        // Digits
        assert_eq!(keyval_to_char(0x30), Some('0'));
        assert_eq!(keyval_to_char(0x39), Some('9'));

        // Space
        assert_eq!(keyval_to_char(0x20), Some(' '));

        // Punctuation
        assert_eq!(keyval_to_char(0x2e), Some('.'));
        assert_eq!(keyval_to_char(0x2c), Some(','));

        // Special keys should return None
        assert_eq!(keyval_to_char(0xff08), None); // Backspace
        assert_eq!(keyval_to_char(0xff0d), None); // Enter
        assert_eq!(keyval_to_char(0xff1b), None); // Escape
    }
}
