use crate::telex;
use crate::utils::{is_shortcut, keyval_to_char};

/// Defines the possible actions the engine can perform in response to a key event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Update the preedit text displayed in the application.
    UpdatePreedit {
        text: String,
        caret: usize,
        visible: bool,
    },
    /// Hide the preedit area.
    HidePreedit,
    /// Send final text to the application.
    Commit(String),
    /// Let the key event pass through to the application without intervention.
    PassThrough,
    /// Prevent the key event from reaching the application because it was handled by the engine.
    Consume,
}

/// Represents the internal state of the input core.
#[derive(Default, Debug)]
pub struct CoreState {
    /// Stores the raw character sequence being processed for Vietnamese transformation.
    pub buffer: String,
}

/// The main logic for handling key events.
/// It decides whether to consume the key, update the preedit, or commit text
/// based on the current buffer and the incoming key.
pub fn handle_key(core: &mut CoreState, keyval: u32, state: u32) -> Vec<Action> {
    // If a shortcut is pressed (e.g., Ctrl+C), pass it through.
    if is_shortcut(state) {
        return vec![Action::PassThrough];
    }

    // Special keys constants (X11 keyvals)
    const BACKSPACE: u32 = 0xff08;
    const RETURN: u32 = 0xff0d;
    const SPACE: u32 = 0x20;

    match keyval {
        BACKSPACE => {
            if core.buffer.is_empty() {
                // Buffer is empty, allow backspace to delete character in the application.
                vec![Action::PassThrough]
            } else {
                // Delete the last character in our internal buffer.
                core.buffer.pop();
                if core.buffer.is_empty() {
                    // Buffer became empty, hide the preedit.
                    vec![Action::HidePreedit, Action::Consume]
                } else {
                    // Transform the remaining buffer and update preedit.
                    let text = vi_transform(&core.buffer);
                    vec![
                        Action::UpdatePreedit {
                            text: text.clone(),
                            caret: text.chars().count(),
                            visible: true,
                        },
                        Action::Consume,
                    ]
                }
            }
        }
        SPACE | RETURN => {
            if core.buffer.is_empty() {
                // Nothing to commit, pass through.
                vec![Action::PassThrough]
            } else {
                // Transform buffer and commit it as a word.
                let text = vi_transform(&core.buffer);
                let mut out = vec![Action::Commit(text), Action::HidePreedit];
                core.buffer.clear();

                // Also commit the space or newline that triggered the commit.
                out.push(Action::Commit(if keyval == SPACE {
                    " ".into()
                } else {
                    "\n".into()
                }));
                out.push(Action::Consume);
                out
            }
        }
        _ => {
            // Check if the key corresponds to a printable character.
            if let Some(ch) = keyval_to_char(keyval) {
                if ch != ' ' {
                    // Add character to buffer and perform Vietnamese transformation.
                    core.buffer.push(ch);
                    let text = vi_transform(&core.buffer);
                    return vec![
                        Action::UpdatePreedit {
                            text: text.clone(),
                            caret: text.chars().count(),
                            visible: true,
                        },
                        Action::Consume,
                    ];
                }
            }
            // Irrelevant keys (e.g., Shift, Caps Lock) are passed through.
            vec![Action::PassThrough]
        }
    }
}

/// Applies Vietnamese input method transformations (Telex) to the given buffer.
pub fn vi_transform(buffer: &str) -> String {
    telex::transform_buffer(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to simulate typing a string into the core.
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
    fn typing_telex_updates_preedit_then_commit_on_space() {
        let mut core = CoreState::default();
        let out = feed(&mut core, "vieetj ");
        assert_eq!(out, "việt ");
        assert_eq!(core.buffer, "");
    }

    #[test]
    fn backspace_edits_buffer() {
        let mut core = CoreState::default();
        let out = {
            let mut committed = String::new();
            for keyval in ['a' as u32, 'b' as u32, 0xff08, ' ' as u32] {
                let actions = handle_key(&mut core, keyval, 0);
                for a in actions {
                    if let Action::Commit(x) = a {
                        committed.push_str(&x);
                    }
                }
            }
            committed
        };
        assert_eq!(out, "a ");
        assert_eq!(core.buffer, "");
    }

    #[test]
    fn passthrough_when_empty_and_space() {
        let mut core = CoreState::default();
        let actions = handle_key(&mut core, ' ' as u32, 0);
        assert!(actions.contains(&Action::PassThrough));
    }

    #[test]
    fn shortcut_passthrough() {
        let mut core = CoreState::default();
        let actions = handle_key(&mut core, 'c' as u32, 1 << 2);
        assert!(actions.contains(&Action::PassThrough));
    }

    #[test]
    fn test_backspace_sequence() {
        let mut core = CoreState::default();
        // Type "vi", then backspace, then "ệt"
        feed(&mut core, "vi");
        handle_key(&mut core, 0xff08, 0); // Backspace 'i' -> buffer 'v'
        let out = feed(&mut core, "eetj ");
        assert_eq!(out, "vệt ");
    }

    #[test]
    fn test_multi_word_typing() {
        let mut core = CoreState::default();
        let out = feed(&mut core, "chaof banj ");
        assert_eq!(out, "chào bạn ");
    }
}
