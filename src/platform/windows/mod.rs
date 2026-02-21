//! Windows keyboard hook backend.
//!
//! This module provides the Windows input method using a low-level
//! keyboard hook (`WH_KEYBOARD_LL`) and `SendInput` for injecting
//! Unicode characters. Runs as a background process.

pub mod engine;

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    WH_KEYBOARD_LL,
};
use tray_item::{IconSource, TrayItem};

/// Starts the Windows keyboard hook and runs the message loop.
///
/// This function:
/// 1. Installs a global low-level keyboard hook
/// 2. Runs the Windows message loop (blocks forever)
/// 3. Cleans up the hook on exit
pub fn run() -> anyhow::Result<()> {
    // Install the low-level keyboard hook
    let hook = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(engine::keyboard_hook_proc),
            None, // hInstance: None for global hooks on the current thread
            0,    // dwThreadId: 0 for global hook
        )?
    };

    eprintln!("XKey Windows running. Press Ctrl+C to exit.");

    // Set up Ctrl+C handler to gracefully exit
    let hook_raw = hook.0 as usize;
    ctrlc::set_handler(move || {
        unsafe {
            let _ = UnhookWindowsHookEx(windows::Win32::UI::WindowsAndMessaging::HHOOK(
                hook_raw as *mut _,
            ));
        }
        std::process::exit(0);
    })
    .expect("Failed to set Ctrl+C handler");

    // Initialize System Tray
    let mut tray = TrayItem::new("XKey", IconSource::Resource("tray-default"))
        .map_err(|e| anyhow::anyhow!("Tray error: {}", e))?;
    
    tray.add_label("XKey Vietnamese Input")
        .map_err(|e| anyhow::anyhow!("{}", e))?;
        
    let quit_hook_raw = hook_raw;
    tray.add_menu_item("Quit", move || {
        unsafe {
            let _ = UnhookWindowsHookEx(windows::Win32::UI::WindowsAndMessaging::HHOOK(
                quit_hook_raw as *mut _,
            ));
        }
        std::process::exit(0);
    })
    .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Run the Windows message loop — this keeps the hook alive
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // Cleanup
    unsafe {
        let _ = UnhookWindowsHookEx(hook);
    }

    Ok(())
}
