//! macOS InputMethodKit engine backend.
//!
//! This module provides the macOS input method using Apple's InputMethodKit
//! framework via `objc2-input-method-kit` bindings.

pub mod engine;

use objc2::rc::Retained;
use objc2::AllocAnyThread;
use objc2_app_kit::NSApplication;
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_input_method_kit::IMKServer;

/// Starts the macOS InputMethodKit server and runs the application run loop.
///
/// This function:
/// 1. Creates an IMKServer with connection name matching Info.plist
/// 2. Starts the NSApplication run loop (blocks forever)
///
/// The IMKServer automatically creates XKeyInputController instances
/// for each input session requested by client applications.
pub fn run() -> anyhow::Result<()> {
    // Register our custom input controller class
    engine::register_class();

    // We must be on the main thread for NSApplication
    let mtm = MainThreadMarker::new()
        .expect("Must be called from the main thread");

    let app = NSApplication::sharedApplication(mtm);

    let connection_name = NSString::from_str("com.manhpham.inputmethod.xkey");
    let bundle_id = NSString::from_str("com.manhpham.inputmethod.xkey");

    // Create the IMK server using the proper Retained API
    let _server: Option<Retained<IMKServer>> = unsafe {
        IMKServer::initWithName_bundleIdentifier(
            IMKServer::alloc(),
            Some(&connection_name),
            Some(&bundle_id),
        )
    };

    let _server = _server.expect("Failed to create IMKServer");

    eprintln!("XKey macOS InputMethod running.");

    // Run the application event loop (blocks forever)
    app.run();

    Ok(())
}
