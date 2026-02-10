//! Core Input Processing Logic for XKey
//!
//! This module handles the core input processing logic, including:
//! - Buffer management for collecting keystrokes
//! - Key event handling and action generation
//! - Integration with the Telex transformation engine
//!
//! The core is designed to be independent of the IBus interface, making it
//! testable in isolation and reusable for other input method backends.

use crate::telex;
use crate::utils::{is_shortcut, keyval_to_char};

/// Actions that the core engine returns for the IBus engine to execute.
///
/// These actions represent the possible responses to a key event. The IBus
/// engine translates these actions into D-Bus signals or other operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Update the preedit (composition) window with new text.
    ///
    /// # Fields
    /// - `text`: The transformed Vietnamese text to display
    /// - `caret`: Cursor position (character index from start)
    /// - `visible`: Whether the preedit window should be shown
    UpdatePreedit {
        text: String,
        caret: usize,
        visible: bool,
    },

    /// Hide the preedit (composition) window.
    /// Typically used when the buffer is cleared or composition is cancelled.
    HidePreedit,

    /// Commit finalized text to the target application.
    /// The text has been fully transformed and should be inserted.
    Commit(String),

    /// Pass the key event through to the application.
    /// The input method did not handle this key.
    PassThrough,

    /// Consume the key event (prevent it from reaching the application).
    /// The input method fully handled this key.
    Consume,
}

/// The core state for input processing.
///
/// This struct maintains the current input buffer which collects raw
/// keystrokes before they are transformed into Vietnamese text.
#[derive(Default, Debug)]
pub struct CoreState {
    /// Raw input buffer containing Telex keystrokes.
    ///
    /// Characters are accumulated here as the user types. The buffer is
    /// transformed in real-time using Telex rules to produce Vietnamese text.
    /// The buffer is cleared when:
    /// - The user presses Space, Enter, or Tab (commits the word)
    /// - The user presses Escape (cancels the composition)
    /// - The user types punctuation (commits the word, then passes punctuation through)
    /// - Focus is lost
    pub buffer: String,
}

/// Processes a key event and returns a list of actions to perform.
///
/// This is the main entry point for key event processing. It handles:
/// - Key release events (ignored)
/// - Shortcut keys (Ctrl+X, Alt+Tab, etc.) - passed through
/// - Backspace - removes last character from buffer
/// - Escape - cancels current composition
/// - Space/Enter/Tab - commits current word
/// - Punctuation - commits current word, then passes through
/// - Regular characters - adds to buffer and updates preedit
///
/// # Arguments
/// * `core` - Mutable reference to the core state
/// * `keyval` - X11 keysym value for the pressed key
/// * `state` - Modifier state bitmask (Shift, Ctrl, Alt, etc.)
///
/// # Returns
/// A vector of actions that the engine should execute in order
///
/// # Key Constants
/// Key values follow X11 keysym conventions:
/// - ASCII characters: 0x20-0x7f (e.g., 'a' = 0x61)
/// - Special keys: 0xff00-0xffff (e.g., Backspace = 0xff08)
pub fn handle_key(core: &mut CoreState, keyval: u32, state: u32) -> Vec<Action> {
    // Ignore key release events (only process key press)
    // IBus sets bit 30 in the state when it's a release event
    const IBUS_RELEASE_MASK: u32 = 1 << 30;
    if state & IBUS_RELEASE_MASK != 0 {
        return vec![Action::PassThrough];
    }

    // Pass through system shortcuts (Ctrl+X, Alt+Tab, Super+..., etc.)
    // These should be handled by the window manager, not the input method
    if is_shortcut(state) {
        return vec![Action::PassThrough];
    }

    // X11 keysym constants for special keys
    const BACKSPACE: u32 = 0xff08; // Delete previous character
    const RETURN: u32 = 0xff0d; // Enter key
    const SPACE: u32 = 0x20; // Space bar
    const TAB: u32 = 0xff09; // Tab key
    const ESC: u32 = 0xff1b; // Escape key

    match keyval {
        // ===== BACKSPACE: Delete the last character from buffer =====
        BACKSPACE => {
            if core.buffer.is_empty() {
                // Nothing to delete, let the app handle it
                vec![Action::PassThrough]
            } else {
                // Remove the last raw character from buffer
                core.buffer.pop();
                if core.buffer.is_empty() {
                    // Buffer is now empty, hide the preedit window
                    vec![Action::HidePreedit, Action::Consume]
                } else {
                    // Update preedit with remaining transformed text
                    let text = vi_transform(&core.buffer);
                    let caret = text.chars().count();
                    vec![
                        Action::UpdatePreedit {
                            text,
                            caret,
                            visible: true,
                        },
                        Action::Consume,
                    ]
                }
            }
        }

        // ===== ESCAPE: Cancel the current composition =====
        ESC => {
            if core.buffer.is_empty() {
                // No active composition, let the app handle Escape
                vec![Action::PassThrough]
            } else {
                // Clear the buffer and hide preedit (discard uncommitted text)
                core.buffer.clear();
                vec![Action::HidePreedit, Action::Consume]
            }
        }

        // ===== WORD TERMINATORS: Space, Enter, Tab =====
        // These keys commit the current word and pass through to the application
        // so their original semantics are preserved (e.g., Enter creates a new line)
        SPACE | RETURN | TAB => {
            if core.buffer.is_empty() {
                // No active composition, pass through directly
                vec![Action::PassThrough]
            } else {
                // Transform and commit the buffered text
                let text = vi_transform(&core.buffer);
                core.buffer.clear();
                vec![
                    Action::Commit(text),
                    Action::HidePreedit,
                    // IMPORTANT: Pass the key through so Space/Enter/Tab
                    // still have their normal effect in the application
                    Action::PassThrough,
                ]
            }
        }

        // ===== ALL OTHER KEYS =====
        _ => {
            if let Some(ch) = keyval_to_char(keyval) {
                // Check if this is a punctuation/separator character
                // These characters commit the current word before being passed through
                let is_separator = matches!(
                    ch,
                    '.' | ',' | ';' | ':' | '!' | '?' | '(' | ')' | '"' | '\'' | '/' | '0'..='9'
                );

                if is_separator {
                    if core.buffer.is_empty() {
                        // No active composition, just pass the punctuation through
                        return vec![Action::PassThrough];
                    }
                    // Commit the current word, then pass the punctuation through
                    let text = vi_transform(&core.buffer);
                    core.buffer.clear();
                    return vec![
                        Action::Commit(text),
                        Action::HidePreedit,
                        Action::PassThrough,
                    ];
                }

                // Regular character: add to buffer and update preedit
                core.buffer.push(ch);
                let text = vi_transform(&core.buffer);
                let caret = text.chars().count();
                return vec![
                    Action::UpdatePreedit {
                        text,
                        caret,
                        visible: true,
                    },
                    Action::Consume,
                ];
            }

            // Unrecognized key (Shift, Caps Lock, arrows, function keys, etc.)
            // Pass through without affecting the composition
            vec![Action::PassThrough]
        }
    }
}

/// Transforms raw Telex input into Vietnamese text.
///
/// This is a thin wrapper around the telex module's transform_buffer function.
/// It applies all Telex transformation rules to convert raw keystrokes into
/// proper Vietnamese characters with diacritics and tone marks.
///
/// # Arguments
/// * `buffer` - Raw Telex input string (e.g., "vieetj")
///
/// # Returns
/// Transformed Vietnamese string (e.g., "việt")
pub fn vi_transform(buffer: &str) -> String {
    telex::transform_buffer(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to simulate typing a string and collect committed text.
    ///
    /// Feeds each character through the key handler and accumulates any
    /// committed text. This is useful for testing complete typing sequences.
    fn feed(core: &mut CoreState, s: &str) -> String {
        let mut committed = String::new();
        for ch in s.chars() {
            let actions = handle_key(core, ch as u32, 0);
            for a in actions {
                if let Action::Commit(x) = a {
                    committed.push_str(&x);
                }
            }
        }
        committed
    }

    #[test]
    fn typing_telex_commit_on_space() {
        // Test: Typing "vieetj " should produce "việt"
        let mut core = CoreState::default();
        let out = feed(&mut core, "vieetj ");
        assert_eq!(out, "việt");
        assert!(core.buffer.is_empty());
    }

    #[test]
    fn backspace_edit() {
        // Test: Typing "ab", then backspace, then space should produce "a"
        let mut core = CoreState::default();
        feed(&mut core, "ab");
        handle_key(&mut core, 0xff08, 0); // Backspace
        let out = feed(&mut core, " ");
        assert_eq!(out, "a");
    }

    #[test]
    fn punctuation_commit() {
        // Test: Typing "chaof?" should commit "chào" when '?' is typed
        let mut core = CoreState::default();
        let out = feed(&mut core, "chaof?");
        assert_eq!(out, "chào");
    }

    #[test]
    fn esc_clear() {
        // Test: Pressing Escape should clear the buffer
        let mut core = CoreState::default();
        feed(&mut core, "vieet");
        handle_key(&mut core, 0xff1b, 0); // Escape
        assert!(core.buffer.is_empty());
    }

    #[test]
    fn digit_separator() {
        // Test: Typing "vieetj6" should commit "việt" when '6' is typed
        let mut core = CoreState::default();
        let out = feed(&mut core, "vieetj6");
        assert_eq!(out, "việt");
        assert!(core.buffer.is_empty());
    }
}
