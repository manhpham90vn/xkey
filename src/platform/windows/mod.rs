//! Windows keyboard hook backend.
//!
//! This module provides the Windows input method using a low-level
//! keyboard hook (`WH_KEYBOARD_LL`) and `SendInput` for injecting
//! Unicode characters. Runs as a background process.

pub mod engine;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tray_item::{IconSource, TrayItem};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    WH_KEYBOARD_LL,
};

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
    let tray = Arc::new(Mutex::new(
        TrayItem::new("XKey", IconSource::Resource("tray-default"))
            .map_err(|e| anyhow::anyhow!("Tray error: {}", e))?,
    ));

    tray.lock()
        .unwrap()
        .add_label("XKey Vietnamese Input")
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let tray_clone = tray.clone();
    let toggle_id = Arc::new(AtomicU32::new(0));
    let toggle_id_clone = toggle_id.clone();

    let initial_label = if engine::ENABLED.load(Ordering::SeqCst) {
        "Bật/Tắt Tiếng Việt (E/V) [Đang Bật]"
    } else {
        "Bật/Tắt Tiếng Việt (E/V) [Đang Tắt]"
    };

    let menu_id = tray
        .lock()
        .unwrap()
        .inner_mut()
        .add_menu_item_with_id(initial_label, move || {
            let current = engine::ENABLED.load(Ordering::SeqCst);
            let new_state = !current;
            engine::ENABLED.store(new_state, Ordering::SeqCst);

            let new_label = if new_state {
                "Bật/Tắt Tiếng Việt (E/V) [Đang Bật]"
            } else {
                "Bật/Tắt Tiếng Việt (E/V) [Đang Tắt]"
            };

            if let Ok(mut t) = tray_clone.lock() {
                let id = toggle_id_clone.load(Ordering::SeqCst);
                let _ = t.inner_mut().set_menu_item_label(new_label, id);
            }
        })
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    toggle_id.store(menu_id, Ordering::SeqCst);

    let quit_hook_raw = hook_raw;
    tray.lock()
        .unwrap()
        .add_menu_item("Quit", move || {
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
