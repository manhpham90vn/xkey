//! XKey - Vietnamese Telex Input Method Engine for IBus
//!
//! This is the main entry point for the xkey application. It can run in two modes:
//! 1. IBus Engine Mode (default): Connects to IBus daemon via D-Bus and serves as an input method
//! 2. REPL Mode (--repl flag): Interactive terminal mode for testing Telex transformations

use tokio::signal;
use zbus::{Address, Connection, ConnectionBuilder};

mod core;
mod engine;
mod repl;
mod telex;
mod utils;

use engine::{FACTORY_OBJ_PATH, OBJ_PATH, XKey, XKeyFactory};
use repl::repl;

/// Retrieves the IBus daemon address for D-Bus connection.
///
/// This function tries two methods to obtain the IBus address:
/// 1. First, it checks the `IBUS_ADDRESS` environment variable, which is typically
///    set by the IBus daemon when launching engine processes.
/// 2. If the environment variable is not available or empty, it falls back to
///    running `ibus address` command to query the address from the IBus CLI tool.
///
/// # Returns
/// - `Ok(String)` containing the IBus D-Bus address on success
/// - `Err` if neither method succeeds in obtaining a valid address
///
/// # Errors
/// - Returns an error if `ibus address` command fails
/// - Returns an error if the command returns an empty address
fn get_ibus_address() -> anyhow::Result<String> {
    // Method 1: Check environment variable (preferred method)
    // IBus daemon typically sets this when launching engine processes
    if let Ok(addr) = std::env::var("IBUS_ADDRESS") {
        let addr = addr.trim();
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }

    // Method 2: Fall back to querying the IBus CLI tool
    // This is useful for manual testing or when running outside IBus control
    let out = std::process::Command::new("ibus").arg("address").output()?;

    anyhow::ensure!(
        out.status.success(),
        "Failed to get IBus address from 'ibus address' command. stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let addr = String::from_utf8(out.stdout)?.trim().to_string();
    anyhow::ensure!(
        !addr.is_empty(),
        "IBus address command returned empty string"
    );
    Ok(addr)
}

/// Main entry point for the xkey application.
///
/// # Modes of Operation
///
/// ## REPL Mode (--repl)
/// When launched with `--repl` flag, starts an interactive terminal session
/// for testing Telex transformations without IBus integration.
///
/// ## IBus Engine Mode (default)
/// When launched without flags (typically by IBus daemon):
/// 1. Creates the XKey engine and factory instances
/// 2. Connects to the IBus daemon via D-Bus
/// 3. Registers the engine at the standard IBus object paths
/// 4. Waits for key events and processes them using Telex rules
/// 5. Gracefully shuts down on Ctrl+C signal
///
/// # D-Bus Registration
/// - Engine is served at: `/org/freedesktop/IBus/Engine/xkey`
/// - Factory is served at: `/org/freedesktop/IBus/Factory`
/// - Bus name: `org.freedesktop.IBus.Engine.xkey`
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Check for REPL mode flag
    // REPL mode is useful for testing Telex transformations in terminal
    if args.iter().any(|a| a == "--repl") {
        return repl();
    }

    // Create the main engine instance that handles key events
    let engine = XKey::new();
    // Create the factory that IBus uses to create engine instances
    let factory = XKeyFactory::new();

    // Get the IBus daemon address and parse it into a D-Bus Address type
    let ibus_addr_str = get_ibus_address()?;
    let ibus_addr = Address::try_from(ibus_addr_str.as_str())?;

    // Build the D-Bus connection with our engine and factory registered
    // The connection builder:
    // - Connects to the IBus daemon at the specified address
    // - Claims the bus name for our engine
    // - Registers the factory and engine at their respective object paths
    let _conn: Connection = ConnectionBuilder::address(ibus_addr)?
        .name("org.freedesktop.IBus.Engine.xkey")?
        .serve_at(FACTORY_OBJ_PATH, factory)?
        .serve_at(OBJ_PATH, engine)?
        .build()
        .await?;

    eprintln!("XKey running. Factory={FACTORY_OBJ_PATH} Engine={OBJ_PATH}");

    // Wait for Ctrl+C signal to gracefully shutdown
    // The engine will continue processing key events until this signal is received
    signal::ctrl_c().await?;
    Ok(())
}
