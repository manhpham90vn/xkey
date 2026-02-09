use tokio::signal;
use zbus::{Connection, ConnectionBuilder};

mod core;
mod engine;
mod repl;
mod telex;
mod utils;

use engine::{OBJ_PATH, XKey};
use repl::repl;

/// Entry point of the XKey application.
/// It can run in either REPL mode for testing or as a D-Bus IBus engine.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Check if the user wants to run in REPL mode for terminal-based testing
    if args.iter().any(|a| a == "--repl") {
        return repl();
    }

    // Initialize the XKey engine which handles the input logic and buffer
    let engine = XKey::new();

    // Set up a session D-Bus connection and register the IBus engine
    // The engine is served at a specific object path (OBJ_PATH)
    // and identified by its well-known name.
    let _conn: Connection = ConnectionBuilder::session()?
        .name("org.freedesktop.IBus.Engine.xkey")?
        .serve_at(OBJ_PATH, engine)?
        .build()
        .await?;

    eprintln!("XKey running. ObjectPath={OBJ_PATH}");

    // Wait for a termination signal (Ctrl+C) to gracefully shut down
    signal::ctrl_c().await?;
    Ok(())
}
