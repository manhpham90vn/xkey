//! REPL (Read-Eval-Print Loop) Mode for XKey
//!
//! This module provides an interactive terminal mode for testing Telex transformations
//! without requiring IBus integration. It's useful for:
//! - Development and debugging of Telex rules
//! - Quick testing of input transformations
//! - Demonstrating how the input method works
//!
//! # Usage
//! Run the application with the `--repl` flag:
//! ```bash
//! cargo run -- --repl
//! ```
//!
//! Then type Vietnamese using Telex and press Enter to see the result.
//! Press Ctrl+D to exit.

use crate::core::{Action, CoreState, handle_key};
use std::io::{self, BufRead};

/// Starts a terminal-based Read-Eval-Print Loop (REPL) for testing Telex transformation.
///
/// This mode allows users to type text directly into the terminal and see how XKey
/// processes it without needing to run as a full IBus engine. It's particularly
/// useful for:
/// - Testing new Telex rules during development
/// - Debugging transformation issues
/// - Demonstrating the input method to users
///
/// # How It Works
/// 1. Reads input line by line from stdin
/// 2. Feeds each character through the core key handler
/// 3. Simulates Enter at the end of each line to commit
/// 4. Displays preedit updates (to stderr) and committed text (to stdout)
///
/// # Output Streams
/// - **stderr**: Preedit updates and status messages (for real-time feedback)
/// - **stdout**: Final committed text (can be piped/redirected)
///
/// # Example Session
/// ```text
/// $ cargo run -- --repl
/// REPL: Type text and press Enter. (Space/Enter will commit). Ctrl+D to exit.
/// vieetj nam
/// preedit='v' caret=1
/// preedit='vi' caret=2
/// preedit='viê' caret=3
/// preedit='việ' caret=3
/// preedit='việt' caret=4
/// commit='việt'
/// preedit hidden
/// preedit='n' caret=1
/// preedit='na' caret=2
/// preedit='nam' caret=3
/// commit='nam'
/// Current committed: 'việt nam'
/// ^D
/// === FINAL RESULT ===
/// việt nam
/// ```
///
/// # Returns
/// * `Ok(())` on successful completion (Ctrl+D pressed)
/// * `Err` if there's an I/O error reading from stdin
pub fn repl() -> anyhow::Result<()> {
    eprintln!("REPL: Type text and press Enter. (Space/Enter will commit). Ctrl+D to exit.");

    let mut core = CoreState::default();
    let mut committed = String::new();

    let stdin = io::stdin();

    // Read input line by line from stdin
    for line in stdin.lock().lines() {
        let line = line?;

        // Feed each character from the line into the core input handler
        for ch in line.chars() {
            let actions = handle_key(&mut core, ch as u32, 0);
            process_repl_actions(&mut committed, actions);
        }

        // Simulate an Enter key press at the end of each line
        // This commits any remaining text in the buffer
        // Enter key is represented by X11 keysym 0xff0d (XK_Return)
        let actions = handle_key(&mut core, 0xff0d, 0);
        process_repl_actions(&mut committed, actions);

        // Show the accumulated committed text after each line
        eprintln!("Current committed: '{}'", committed);
    }

    // Print final result to stdout (can be piped/redirected)
    eprintln!("\n=== FINAL RESULT ===");
    println!("{}", committed);

    Ok(())
}

/// Processes the actions returned by the core engine for the REPL environment.
///
/// This function handles the Action enum values and provides appropriate
/// feedback for the terminal environment. Unlike the IBus engine which
/// sends D-Bus signals, the REPL prints updates to stderr for visibility.
///
/// # Arguments
/// * `committed` - Mutable string to accumulate committed text
/// * `actions` - List of actions from the core key handler
///
/// # Action Handling
/// - `UpdatePreedit`: Logs the current composition state to stderr
/// - `HidePreedit`: Logs that the preedit was hidden
/// - `Commit`: Appends text to the committed string and logs it
/// - `Consume`/`PassThrough`: Ignored in REPL mode
fn process_repl_actions(committed: &mut String, actions: Vec<Action>) {
    for a in actions {
        match a {
            Action::UpdatePreedit { text, caret, .. } => {
                // Log preedit updates to stderr to distinguish from final output
                // This shows the user the current composition in real-time
                eprintln!("preedit='{}' caret={}", text, caret);
            }
            Action::HidePreedit => {
                // Notify that the composition window would be hidden
                eprintln!("preedit hidden");
            }
            Action::Commit(s) => {
                // Accumulate committed text and log it
                // Replace newline with escaped version for clear logging
                committed.push_str(&s);
                eprintln!("commit='{}'", s.replace('\n', "\\n"));
            }
            // Consume and PassThrough don't need special handling in REPL
            // They're only relevant for the IBus engine's return value
            _ => {}
        }
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_repl_actions_commit() {
        let mut committed = String::new();
        let actions = vec![Action::Commit("test".to_string())];
        process_repl_actions(&mut committed, actions);
        assert_eq!(committed, "test");
    }

    #[test]
    fn test_process_repl_actions_multiple() {
        let mut committed = String::new();
        let actions = vec![
            Action::Commit("hello".to_string()),
            Action::Commit(" ".to_string()),
            Action::Commit("world".to_string()),
        ];
        process_repl_actions(&mut committed, actions);
        assert_eq!(committed, "hello world");
    }
}
