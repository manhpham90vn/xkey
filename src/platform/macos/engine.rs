//! macOS InputMethodKit Engine Implementation
//!
//! This module implements the macOS input method controller using Apple's
//! InputMethodKit framework. It defines `XKeyInputController` which inherits
//! from `IMKInputController` and handles key events, preedit text, and
//! text commitment using the platform-independent core engine.

use crate::core::{Action, CoreState, handle_key};
use std::cell::RefCell;
use std::collections::HashMap;

use objc2::runtime::{AnyObject, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel};
use objc2_foundation::{NSRange, NSString};

// Per-controller CoreState, keyed by the controller's pointer address.
// Since macOS creates one controller per input session and they all run
// on the main thread, a thread-local HashMap is safe and efficient.
thread_local! {
    static CORES: RefCell<HashMap<usize, CoreState>> = RefCell::new(HashMap::new());
}

/// Helper to get or create a CoreState for a controller instance.
fn with_core<F, R>(controller: *mut AnyObject, f: F) -> R
where
    F: FnOnce(&mut CoreState) -> R,
{
    let key = controller as usize;
    CORES.with(|cores| {
        let mut map = cores.borrow_mut();
        let core = map.entry(key).or_insert_with(CoreState::default);
        f(core)
    })
}

/// Remove the CoreState for a controller instance (cleanup on dealloc).
fn remove_core(controller: *mut AnyObject) {
    let key = controller as usize;
    CORES.with(|cores| {
        cores.borrow_mut().remove(&key);
    });
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
///
/// Uses NSNotFound for replacementRange location in all client calls so that
/// text is always inserted/replaced at the current cursor position.
fn apply_actions(client: *mut AnyObject, actions: Vec<Action>) -> bool {
    // NSNotFound on 64-bit macOS = NSIntegerMax = isize::MAX.
    // Using this for replacementRange location tells the system to use
    // the current insertion point rather than a specific document offset.
    let not_found = NSRange::new(isize::MAX as usize, isize::MAX as usize);

    let mut consumed = false;
    let mut just_committed = false;

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
                    let replace_range = not_found;
                    unsafe {
                        let _: () = msg_send![
                            client,
                            setMarkedText: &*ns_text,
                            selectionRange: sel_range,
                            replacementRange: replace_range
                        ];
                    }
                }
                just_committed = false;
            }

            Action::HidePreedit => {
                if !just_committed {
                    unsafe {
                        let empty = NSString::from_str("");
                        let sel_range = NSRange::new(0, 0);
                        let replace_range = not_found;
                        let _: () = msg_send![
                            client,
                            setMarkedText: &*empty,
                            selectionRange: sel_range,
                            replacementRange: replace_range
                        ];
                    }
                }
                just_committed = false;
            }

            Action::Commit(s) => {
                let ns_text = NSString::from_str(&s);
                let replace_range = not_found;
                unsafe {
                    let _: () = msg_send![
                        client,
                        insertText: &*ns_text,
                        replacementRange: replace_range
                    ];
                }
                just_committed = true;
            }

            Action::Consume => consumed = true,
            Action::PassThrough => {
                just_committed = false;
            }
        }
    }

    consumed
}

// ============================================================================
// Objective-C class registration
// ============================================================================

/// The handleEvent:client: method implementation.
///
/// This is the correct InputMethodKit protocol method that is called by the
/// system when a key event occurs. It receives an NSEvent and the client object.
///
/// Signature: -(BOOL)handleEvent:(NSEvent*)event client:(id)sender
extern "C" fn handle_event(
    this: *mut AnyObject,
    _cmd: Sel,
    event: *mut AnyObject,
    client: *mut AnyObject,
) -> Bool {
    // Get modifier flags
    let flags: usize = unsafe { msg_send![event, modifierFlags] };
    let state = modifiers_to_state(flags as u64);

    // Get event type to distinguish key down/up/flags changed
    let event_type: u64 = unsafe { msg_send![event, type] };
    // NSEventType: keyDown = 10, keyUp = 11, flagsChanged = 12
    if event_type != 10 {
        // Only process key down events
        return Bool::NO;
    }

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

    let actions = with_core(this, |core| handle_key(core, keyval, state));

    let consumed = apply_actions(client, actions);
    if consumed { Bool::YES } else { Bool::NO }
}

/// The activateServer: method implementation.
///
/// Called when the input method is activated for a client.
/// We reset the CoreState for this controller to ensure clean state.
///
/// Signature: -(void)activateServer:(id)sender
extern "C" fn activate_server(this: *mut AnyObject, _cmd: Sel, _sender: *mut AnyObject) {
    // Reset buffer when switching to this input method
    with_core(this, |core| {
        core.buffer.clear();
    });
}

/// The deactivateServer: method implementation.
///
/// Called when the input method is deactivated.
/// We commit any pending text and clear the buffer.
///
/// Signature: -(void)deactivateServer:(id)sender
extern "C" fn deactivate_server(this: *mut AnyObject, _cmd: Sel, sender: *mut AnyObject) {
    // Commit any pending text before deactivating
    let actions = with_core(this, |core| {
        if core.buffer.is_empty() {
            return vec![];
        }
        let text = crate::core::vi_transform(&core.buffer);
        core.buffer.clear();
        vec![Action::Commit(text), Action::HidePreedit]
    });

    if !actions.is_empty() {
        apply_actions(sender, actions);
    }
}

/// The inputControllerWillClose callback.
/// Cleans up the per-controller CoreState from the HashMap.
extern "C" fn input_controller_will_close(this: *mut AnyObject, _cmd: Sel) {
    remove_core(this);
}

/// Register the XKeyInputController class with the Objective-C runtime.
pub fn register_class() {
    let superclass = class!(IMKInputController);
    let name = c"XKeyInputController";
    let mut builder =
        ClassBuilder::new(name, superclass).expect("Failed to create XKeyInputController class");

    unsafe {
        // handleEvent:client: — the primary key event handler for IMK
        builder.add_method(
            sel!(handleEvent:client:),
            handle_event
                as extern "C" fn(*mut AnyObject, Sel, *mut AnyObject, *mut AnyObject) -> Bool,
        );

        // activateServer: — called when input method is activated
        builder.add_method(
            sel!(activateServer:),
            activate_server as extern "C" fn(*mut AnyObject, Sel, *mut AnyObject),
        );

        // deactivateServer: — called when input method is deactivated
        builder.add_method(
            sel!(deactivateServer:),
            deactivate_server as extern "C" fn(*mut AnyObject, Sel, *mut AnyObject),
        );

        // inputControllerWillClose — cleanup
        builder.add_method(
            sel!(inputControllerWillClose),
            input_controller_will_close as extern "C" fn(*mut AnyObject, Sel),
        );
    }

    builder.register();
}
