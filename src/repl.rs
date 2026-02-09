use crate::core::{Action, CoreState, handle_key};
use std::io::{self, BufRead};

/// Starts a terminal-based Read-Eval-Print Loop (REPL) for testing Telex transformation.
/// This mode allows users to type text directly into the terminal and see how XKey
/// processes it without needing to run as a full IBus engine.
pub fn repl() -> anyhow::Result<()> {
    eprintln!("REPL: Type text and press Enter. (Space/Enter will commit). Ctrl+D to exit.");
    let mut core = CoreState::default();
    let mut committed = String::new();

    let stdin = io::stdin();
    // Read input line by line
    for line in stdin.lock().lines() {
        let line = line?;
        // Feed each character from the line into the core input handler
        for ch in line.chars() {
            let actions = handle_key(&mut core, ch as u32, 0);
            process_repl_actions(&mut committed, actions);
        }
        // Simulate an Enter key press at the end of each line to commit the last word
        let actions = handle_key(&mut core, 0xff0d, 0);
        process_repl_actions(&mut committed, actions);

        eprintln!("Current committed: '{}'", committed);
    }

    eprintln!("\n=== FINAL RESULT ===");
    println!("{}", committed);
    Ok(())
}

/// Processes the actions returned by the core engine specifically for the REPL environment.
fn process_repl_actions(committed: &mut String, actions: Vec<Action>) {
    for a in actions {
        match a {
            Action::UpdatePreedit { text, caret, .. } => {
                // Log preedit updates to stderr to distinguish from final output
                eprintln!("preedit='{}' caret={}", text, caret);
            }
            Action::HidePreedit => {
                eprintln!("preedit hidden");
            }
            Action::Commit(s) => {
                // Collect committed text to display later
                committed.push_str(&s);
                eprintln!("commit='{}'", s.replace('\n', "\\n"));
            }
            _ => {}
        }
    }
}
