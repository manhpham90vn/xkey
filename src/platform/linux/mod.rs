//! Linux IBus engine backend.
//!
//! This module provides the IBus input method engine that connects
//! to the IBus daemon via D-Bus and processes key events.

pub mod engine;

use engine::{FACTORY_OBJ_PATH, OBJ_PATH, XKey, XKeyFactory};
use zbus::{Address, Connection, ConnectionBuilder};

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
fn get_ibus_address() -> anyhow::Result<String> {
    // Method 1: Check environment variable (preferred method)
    if let Ok(addr) = std::env::var("IBUS_ADDRESS") {
        let addr = addr.trim();
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }

    // Method 2: Fall back to querying the IBus CLI tool
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

/// Starts the IBus engine and runs until Ctrl+C.
///
/// This function:
/// 1. Creates the XKey engine and factory instances
/// 2. Connects to the IBus daemon via D-Bus
/// 3. Registers the engine at the standard IBus object paths
/// 4. Waits for Ctrl+C signal to gracefully shutdown
pub async fn run() -> anyhow::Result<()> {
    let engine = XKey::new();
    let factory = XKeyFactory::new();

    let ibus_addr_str = get_ibus_address()?;
    let ibus_addr = Address::try_from(ibus_addr_str.as_str())?;

    let _conn: Connection = ConnectionBuilder::address(ibus_addr)?
        .name("org.freedesktop.IBus.Engine.xkey")?
        .serve_at(FACTORY_OBJ_PATH, factory)?
        .serve_at(OBJ_PATH, engine)?
        .build()
        .await?;

    eprintln!("XKey running. Factory={FACTORY_OBJ_PATH} Engine={OBJ_PATH}");

    tokio::signal::ctrl_c().await?;
    Ok(())
}
