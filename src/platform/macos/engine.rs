//! macOS InputMethodKit Engine Implementation
//!
//! This module implements the macOS input method controller using Apple's
//! InputMethodKit framework. It defines `XKeyInputController` which inherits
//! from `IMKInputController` and handles key events, preedit text, and
//! text commitment using the platform-independent core engine.

use crate::core::{Action, CoreState, handle_key};
use std::cell::RefCell;

use objc2::runtime::{AnyObject, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel};
use objc2_foundation::{NSRange, NSString};

// Thread-local core state for each input controller instance.
// Since macOS creates one controller per input session and they run
// on the main thread, thread_local is safe and avoids Mutex overhead.
thread_local! {
    static CORE: RefCell<CoreState> = RefCell::new(CoreState::default());
}

/// macOS modifier flag masks (from NSEvent.modifierFlags)
const NS_COMMAND_KEY_MASK: u64 = 1 << 20;
const NS_CONTROL_KEY_MASK: u64 = 1 << 18;
const NS_ALTERNATE_KEY_MASK: u64 = 1 << 19;

/// Converts macOS modifier flags to the state bitmask format used by core::handle_key.
fn modifiers_to_state(flags: u64) -> u32 {
    let mut state: u32 = 0;
    if flags & NS_CONTROL_KEY_MASK != 0 {
        state |= 1 << 2; // CTRL
    }
    if flags & NS_ALTERNATE_KEY_MASK != 0 {
        state |= 1 << 3; // ALT
    }
    if flags & NS_COMMAND_KEY_MASK != 0 {
        state |= 1 << 26; // SUPER/CMD
    }
    state
}



/// Applies actions returned by core::handle_key to the macOS input client.
fn apply_actions(client: *mut AnyObject, actions: Vec<Action>) -> bool {
    let mut consumed = false;

    for action in actions {
        match action {
            Action::UpdatePreedit {
                text,
                caret,
                visible,
            } => {
                if visible {
                    let ns_text = NSString::from_str(&text);
                    let sel_range = NSRange::new(caret, 0);
                    let replace_range = NSRange::new(0, 0);
                    unsafe {
                        let _: () = msg_send![
                            client,
                            setMarkedText: &*ns_text,
                            selectionRange: sel_range,
                            replacementRange: replace_range
                        ];
                    }
                }
            }

            Action::HidePreedit => unsafe {
                let empty = NSString::from_str("");
                let sel_range = NSRange::new(0, 0);
                let replace_range = NSRange::new(0, 0);
                let _: () = msg_send![
                    client,
                    setMarkedText: &*empty,
                    selectionRange: sel_range,
                    replacementRange: replace_range
                ];
            },

            Action::Commit(s) => {
                let ns_text = NSString::from_str(&s);
                let replace_range = NSRange::new(usize::MAX, usize::MAX);
                unsafe {
                    let _: () = msg_send![
                        client,
                        insertText: &*ns_text,
                        replacementRange: replace_range
                    ];
                }
            }

            Action::Consume => consumed = true,
            Action::PassThrough => {}
        }
    }

    consumed
}

// ============================================================================
// Objective-C class registration
// ============================================================================

/// The handle:client: method implementation.
extern "C" fn handle_event(
    _this: *mut AnyObject,
    _cmd: Sel,
    event: *mut AnyObject,
    client: *mut AnyObject,
) -> Bool {
    // Get modifier flags
    let flags: usize = unsafe { msg_send![event, modifierFlags] };
    let state = modifiers_to_state(flags as u64);

    // Get key code
    let key_code: u16 = unsafe { msg_send![event, keyCode] };

    // Map special keys
    let keyval: Option<u32> = match key_code {
        51 => Some(0xff08), // Backspace
        36 => Some(0xff0d), // Return
        48 => Some(0xff09), // Tab
        53 => Some(0xff1b), // Escape
        49 => Some(0x20),   // Space
        _ => {
            // For printable characters, get characters string
            let chars: *mut AnyObject = unsafe { msg_send![event, characters] };
            if !chars.is_null() {
                let ns_str: &NSString = unsafe { &*(chars as *const NSString) };
                let s = ns_str.to_string();
                s.chars().next().and_then(|ch| {
                    if ch.is_ascii_graphic() || ch == ' ' {
                        Some(ch as u32)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        }
    };

    let keyval = match keyval {
        Some(kv) => kv,
        None => return Bool::NO,
    };

    let actions = CORE.with(|core| {
        let mut core = core.borrow_mut();
        handle_key(&mut core, keyval, state)
    });

    let consumed = apply_actions(client, actions);
    if consumed {
        Bool::YES
    } else {
        Bool::NO
    }
}

/// Register the XKeyInputController class with the Objective-C runtime.
pub fn register_class() {
    let superclass = class!(IMKInputController);
    let name = c"XKeyInputController";
    let mut builder = ClassBuilder::new(name, superclass)
        .expect("Failed to create XKeyInputController class");

    unsafe {
        builder.add_method(
            sel!(handle:client:),
            handle_event
                as extern "C" fn(*mut AnyObject, Sel, *mut AnyObject, *mut AnyObject) -> Bool,
        );
    }

    builder.register();
}

