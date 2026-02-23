//! Windows Keyboard Hook Engine Implementation
//!
//! This module implements the Windows input method using a low-level keyboard
//! hook (`WH_KEYBOARD_LL`) and `SendInput` for injecting Unicode characters.
//! This is the same approach used by popular Vietnamese input methods like
//! UniKey and EVKey.

use crate::core::{Action, CoreState, handle_key};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_ESCAPE, VK_LWIN, VK_MENU,
    VK_RETURN, VK_RWIN, VK_SPACE, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_SYSKEYDOWN,
};

// Thread-local core state for the keyboard hook.
// The hook callback runs on the thread that installed the hook,
// so thread_local is safe.
thread_local! {
    static CORE: RefCell<CoreState> = RefCell::new(CoreState::default());
}

/// Flag to prevent re-entrance when we inject keystrokes via SendInput.
/// When we call SendInput, it triggers the keyboard hook again — we need
/// to let those injected events pass through.
static SENDING: AtomicBool = AtomicBool::new(false);

/// Global switch to enable/disable Vietnamese input method.
pub static ENABLED: AtomicBool = AtomicBool::new(true);

// Track the string currently displayed as preedit.
// We use this to calculate the common prefix between the old text
// and the new text, only sending backspaces for the difference.
thread_local! {
    static PREEDIT_TEXT: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Checks if a modifier key (Ctrl, Alt, Win) is currently held down.
fn is_modifier_held() -> u32 {
    let mut state: u32 = 0;
    unsafe {
        if GetAsyncKeyState(VK_CONTROL.0 as i32) < 0 {
            state |= 1 << 2; // CTRL
        }
        if GetAsyncKeyState(VK_MENU.0 as i32) < 0 {
            state |= 1 << 3; // ALT
        }
        if GetAsyncKeyState(VK_LWIN.0 as i32) < 0 || GetAsyncKeyState(VK_RWIN.0 as i32) < 0 {
            state |= 1 << 26; // SUPER/WIN
        }
    }
    state
}

/// Converts a Windows virtual key code to the X11 keysym-style keyval
/// used by `core::handle_key()`.
fn vk_to_keyval(vk_code: u32, scan_code: u32) -> Option<u32> {
    match VIRTUAL_KEY(vk_code as u16) {
        VK_BACK => Some(0xff08),   // Backspace
        VK_RETURN => Some(0xff0d), // Enter
        VK_TAB => Some(0xff09),    // Tab
        VK_ESCAPE => Some(0xff1b), // Escape
        VK_SPACE => Some(0x20),    // Space
        _ => {
            // Convert virtual key to Unicode character using ToUnicode
            let mut key_state = [0u8; 256];
            unsafe {
                windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardState(&mut key_state)
                    .ok()?;
            }
            let mut buf = [0u16; 4];
            let result = unsafe {
                windows::Win32::UI::Input::KeyboardAndMouse::ToUnicode(
                    vk_code,
                    scan_code,
                    Some(&key_state),
                    &mut buf,
                    0,
                )
            };
            if result == 1 {
                let ch = char::from_u32(buf[0] as u32)?;
                if ch.is_ascii_graphic() || ch == ' ' {
                    Some(ch as u32)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

/// Sends backspace key events to delete `count` characters.
fn send_backspaces(count: usize) {
    if count == 0 {
        return;
    }

    let mut inputs: Vec<INPUT> = Vec::with_capacity(count * 2);
    for _ in 0..count {
        // Key down
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_BACK,
                    wScan: 0,
                    dwFlags: Default::default(),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        // Key up
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_BACK,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }

    SENDING.store(true, Ordering::SeqCst);
    unsafe {
        SendInput(&inputs, size_of::<INPUT>() as i32);
    }
    SENDING.store(false, Ordering::SeqCst);
}

/// Sends a Unicode string by injecting `SendInput` events.
fn send_unicode_string(text: &str) {
    if text.is_empty() {
        return;
    }

    let mut inputs: Vec<INPUT> = Vec::new();
    for ch in text.encode_utf16() {
        // Key down
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: ch,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        // Key up
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: ch,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }

    SENDING.store(true, Ordering::SeqCst);
    unsafe {
        SendInput(&inputs, size_of::<INPUT>() as i32);
    }
    SENDING.store(false, Ordering::SeqCst);
}

/// Applies actions returned by `core::handle_key()`.
///
/// Returns `true` if the original keystroke should be suppressed.
fn apply_actions(actions: Vec<Action>) -> bool {
    let mut consumed = false;

    for action in actions {
        match action {
            Action::UpdatePreedit { text, visible, .. } => {
                if visible {
                    PREEDIT_TEXT.with(|p| {
                        let mut prev_text = p.borrow_mut();

                        // Find common prefix length
                        let mut common_prefix_len = 0;
                        for (c1, c2) in prev_text.chars().zip(text.chars()) {
                            if c1 == c2 {
                                common_prefix_len += 1;
                            } else {
                                break;
                            }
                        }

                        // Backspace the non-matching part of the old text
                        let prev_len = prev_text.chars().count();
                        if prev_len > common_prefix_len {
                            send_backspaces(prev_len - common_prefix_len);
                        }

                        // Send the new suffix
                        let new_suffix: String = text.chars().skip(common_prefix_len).collect();
                        send_unicode_string(&new_suffix);

                        // Update tracking
                        *prev_text = text.clone();
                    });
                }
            }

            Action::HidePreedit => {
                PREEDIT_TEXT.with(|p| {
                    let mut prev_text = p.borrow_mut();
                    let prev_len = prev_text.chars().count();
                    send_backspaces(prev_len);
                    prev_text.clear();
                });
            }

            Action::Commit(text) => {
                PREEDIT_TEXT.with(|p| {
                    let mut prev_text = p.borrow_mut();

                    // Same diff logic as UpdatePreedit to minimize backspaces
                    let mut common_prefix_len = 0;
                    for (c1, c2) in prev_text.chars().zip(text.chars()) {
                        if c1 == c2 {
                            common_prefix_len += 1;
                        } else {
                            break;
                        }
                    }

                    let prev_len = prev_text.chars().count();
                    if prev_len > common_prefix_len {
                        send_backspaces(prev_len - common_prefix_len);
                    }

                    let new_suffix: String = text.chars().skip(common_prefix_len).collect();
                    send_unicode_string(&new_suffix);

                    // Clear tracking since it's committed
                    prev_text.clear();
                });
            }

            Action::Consume => consumed = true,
            Action::PassThrough => {}
        }
    }

    consumed
}

/// The low-level keyboard hook callback.
///
/// This is called by Windows for every keystroke system-wide.
/// We process key-down events, feed them to the core engine, and
/// either suppress or pass through the original keystroke.
///
/// # Safety
/// This function is called by the Windows API as a callback.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "system" fn keyboard_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    // If we're injecting keystrokes, let them pass through
    if SENDING.load(Ordering::SeqCst) {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // Only process if nCode >= 0 (HC_ACTION)
    if n_code < 0 {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // Skip processing if input method is disabled by user
    if !ENABLED.load(Ordering::SeqCst) {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // Only handle key-down events
    let msg = w_param.0 as u32;
    if msg != WM_KEYDOWN && msg != WM_SYSKEYDOWN {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    let kb_struct = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
    let vk_code = kb_struct.vkCode;
    let scan_code = kb_struct.scanCode;

    // Get modifier state
    let state = is_modifier_held();

    // Convert Windows VK to our keyval format
    let keyval = match vk_to_keyval(vk_code, scan_code) {
        Some(kv) => kv,
        None => return CallNextHookEx(None, n_code, w_param, l_param),
    };

    // Process through core engine
    let actions = CORE.with(|core| {
        let mut core = core.borrow_mut();
        handle_key(&mut core, keyval, state)
    });

    let consumed = apply_actions(actions);

    if consumed {
        // Suppress the original keystroke
        LRESULT(1)
    } else {
        CallNextHookEx(None, n_code, w_param, l_param)
    }
}
