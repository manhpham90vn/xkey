//! XKey - Vietnamese Telex Input Method Engine
//!
//! This is the main entry point for the xkey application. It can run in two modes:
//! 1. Input Method Mode (default):
//!    - **Linux**: Connects to IBus daemon via D-Bus
//!    - **macOS**: Runs as InputMethodKit server
//! 2. REPL Mode (--repl flag): Interactive terminal mode for testing Telex transformations

mod core;
mod platform;
mod repl;
mod telex;
mod utils;

use repl::repl;

/// Main entry point for the xkey application.
///
/// # Modes of Operation
///
/// ## REPL Mode (--repl)
/// When launched with `--repl` flag, starts an interactive terminal session
/// for testing Telex transformations without input method integration.
/// Available on all platforms.
///
/// ## Input Method Mode (default)
/// ### Linux
/// Connects to IBus daemon via D-Bus, registers as an IBus engine.
/// ### macOS
/// Creates an IMKServer instance and runs the NSApplication event loop.
#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--repl") {
        return repl();
    }

    platform::linux::run().await
}

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--repl") {
        return repl();
    }

    platform::macos::run()
}
