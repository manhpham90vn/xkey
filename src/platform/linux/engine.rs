//! IBus Engine Implementation for XKey
//!
//! This module implements the IBus Engine interface using D-Bus (zbus).
//! It handles:
//! - Key event processing from IBus daemon
//! - Preedit text display (composition window)
//! - Text commitment to the target application
//! - Engine lifecycle events (focus, reset, enable/disable)

use crate::core::{Action, CoreState, handle_key};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zbus::{interface, object_server::SignalContext};
use zvariant::{OwnedValue, Structure, Value};

/// D-Bus object path where the XKey engine is registered.
/// IBus daemon uses this path to communicate with our engine.
pub const OBJ_PATH: &str = "/org/freedesktop/IBus/Engine/xkey";

/// D-Bus object path for the engine factory.
/// IBus uses this to create new engine instances when needed.
pub const FACTORY_OBJ_PATH: &str = "/org/freedesktop/IBus/Factory";

/// Internal state container for the engine.
/// Wraps the core state that handles Telex transformation logic.
#[derive(Default)]
pub struct EngineState {
    /// The core state that manages the input buffer and Telex transformations
    pub core: CoreState,
}

/// The main IBus engine implementation for XKey.
///
/// This struct is registered on D-Bus and receives method calls from IBus daemon.
/// It uses thread-safe shared state (Arc<Mutex>) to allow concurrent access
/// from async D-Bus handlers.
pub struct XKey {
    /// Thread-safe shared state wrapped in Arc<Mutex> for concurrent access.
    /// This is necessary because D-Bus method handlers may be called from
    /// different async contexts.
    pub st: Arc<Mutex<EngineState>>,
}

impl Default for XKey {
    fn default() -> Self {
        Self::new()
    }
}

impl XKey {
    /// Creates a new XKey engine instance with default state.
    pub fn new() -> Self {
        Self {
            st: Arc::new(Mutex::new(EngineState::default())),
        }
    }

    /// Applies a list of actions returned by the core engine.
    ///
    /// This method processes each action and emits the corresponding D-Bus signals
    /// to communicate with the IBus daemon. Actions include:
    /// - UpdatePreedit: Updates the composition window with current text
    /// - HidePreedit: Hides the composition window
    /// - Commit: Sends finalized text to the target application
    /// - Consume: Indicates the key event was handled (prevents propagation)
    /// - PassThrough: Allows the key event to pass to the application
    ///
    /// # Arguments
    /// * `ctxt` - D-Bus signal context for emitting signals
    /// * `actions` - List of actions to process
    ///
    /// # Returns
    /// * `true` if the key event should be consumed (not passed to application)
    /// * `false` if the key event should pass through
    async fn apply_actions(
        &self,
        ctxt: &SignalContext<'_>,
        actions: Vec<Action>,
    ) -> zbus::fdo::Result<bool> {
        let mut consume = false;

        for action in actions {
            match action {
                Action::UpdatePreedit {
                    text,
                    caret,
                    visible,
                } => {
                    // Send preedit update signal to IBus
                    // This updates the composition window that shows text being typed
                    Self::update_preedit_text(
                        ctxt,
                        ibus_text(text),
                        caret as u32,
                        visible,
                        1, // mode: 1 = IBUS_ENGINE_PREEDIT_COMMIT (auto-commit on focus loss)
                    )
                    .await?;
                }

                Action::HidePreedit => {
                    // Hide the composition window
                    Self::hide_preedit_text(ctxt).await?;
                }

                Action::Commit(s) => {
                    // Send the finalized text to the target application
                    Self::commit_text(ctxt, ibus_text(s)).await?;
                }

                // Mark that we handled this key event
                Action::Consume => consume = true,
                Action::SyncPreedit(text) => {
                    // On IBus, we can't let the OS type the character directly
                    // (PassThrough) and track it silently, because IBus preedit
                    // is separate from committed text. If we let keys pass through
                    // and later switch to UpdatePreedit, the already-typed chars
                    // remain in the app, causing duplication (e.g. "too" → "tôtô").
                    // Instead, show the text as preedit and consume the key.
                    let caret = text.chars().count();
                    Self::update_preedit_text(
                        ctxt,
                        ibus_text(text),
                        caret as u32,
                        true,
                        1, // IBUS_ENGINE_PREEDIT_COMMIT
                    )
                    .await?;
                    consume = true;
                }
                // Allow key to pass through to application
                Action::PassThrough => {}
            }
        }

        Ok(consume)
    }
}

/// D-Bus interface implementation for org.freedesktop.IBus.Engine
///
/// This interface defines the methods that IBus daemon calls to communicate
/// with input method engines. The #[interface] macro from zbus automatically
/// generates the D-Bus method stubs.
#[interface(name = "org.freedesktop.IBus.Engine")]
impl XKey {
    /// Called by IBus when a key event occurs.
    ///
    /// This is the main entry point for processing keyboard input.
    /// It receives the key information, processes it through the core engine,
    /// and returns whether the key was consumed.
    ///
    /// # Arguments
    /// * `keyval` - X11 keysym value (e.g., 0x61 for 'a', 0xff08 for Backspace)
    /// * `_keycode` - Hardware keycode (unused, we use keyval instead)
    /// * `state` - Modifier state (Shift, Ctrl, Alt, etc. as bitmask)
    ///
    /// # Returns
    /// * `true` - Key was consumed (processed by input method)
    /// * `false` - Key should pass through to application
    async fn process_key_event(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        keyval: u32,
        _keycode: u32,
        state: u32,
    ) -> zbus::fdo::Result<bool> {
        // Lock the state, process the key, and release the lock immediately
        // This prevents holding the lock across await points
        let actions = {
            let mut guard = self.st.lock().unwrap_or_else(|e| e.into_inner());
            handle_key(&mut guard.core, keyval, state)
        };

        // Apply the resulting actions (may involve async D-Bus signals)
        self.apply_actions(&ctxt, actions).await
    }

    /// Called when the input focus enters a text field.
    ///
    /// This is a lifecycle event that allows the engine to prepare for input.
    /// Currently, we don't need any special handling on focus in.
    async fn focus_in(
        &self,
        #[zbus(signal_context)] _ctxt: SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        Ok(())
    }

    /// Called when the input focus leaves a text field.
    ///
    /// We clear the input buffer and hide the preedit window when focus is lost.
    /// This ensures a clean state when the user switches to a different field.
    ///
    /// Note: We must drop the mutex guard before calling async methods to avoid
    /// holding the lock across await points (which could cause deadlocks).
    async fn focus_out(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        // Commit any pending preedit text before clearing the buffer
        // This prevents losing the last word when focus changes
        // (e.g., user clicks Send button in browser)
        let pending_text = {
            let mut guard = self.st.lock().unwrap_or_else(|e| e.into_inner());
            if guard.core.buffer.is_empty() {
                None
            } else {
                let text = crate::core::vi_transform(&guard.core.buffer);
                guard.core.buffer.clear();
                Some(text)
            }
        }; // Lock is released here, before the await

        if let Some(text) = pending_text {
            Self::commit_text(&ctxt, ibus_text(text)).await?;
        }
        Self::hide_preedit_text(&ctxt).await?;
        Ok(())
    }

    /// Called to reset the engine state.
    ///
    /// Similar to focus_out, this clears the buffer and hides the preedit.
    /// This is typically called when the user requests a reset or when
    /// switching input methods.
    async fn reset(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        {
            // Lock scope: clear buffer while holding the lock
            let mut guard = self.st.lock().unwrap_or_else(|e| e.into_inner());
            guard.core.buffer.clear();
        } // Lock is released here, before the await

        Self::hide_preedit_text(&ctxt).await?;
        Ok(())
    }

    /// Called when the engine is enabled (activated by user).
    fn enable(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    /// Called when the engine is disabled (deactivated by user).
    fn disable(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    // ===== D-Bus Signals =====
    // These signals are emitted to communicate with IBus daemon.
    // The #[zbus(signal)] attribute generates the signal emission code.

    /// Updates the preedit (composition) window.
    ///
    /// # Arguments
    /// * `text` - IBusText variant containing the preedit string
    /// * `cursor_pos` - Cursor position within the preedit text
    /// * `visible` - Whether the preedit window should be visible
    /// * `mode` - Preedit mode (0 = clear on focus out)
    #[zbus(signal)]
    pub async fn update_preedit_text(
        ctxt: &SignalContext<'_>,
        text: Value<'_>, // IBusText as D-Bus variant
        cursor_pos: u32, // Cursor position (character index)
        visible: bool,   // Visibility flag
        mode: u32,       // Preedit mode
    ) -> zbus::Result<()>;

    /// Hides the preedit window.
    #[zbus(signal)]
    pub async fn hide_preedit_text(ctxt: &SignalContext<'_>) -> zbus::Result<()>;

    /// Commits finalized text to the target application.
    ///
    /// # Arguments
    /// * `text` - IBusText variant containing the text to commit
    #[zbus(signal)]
    pub async fn commit_text(ctxt: &SignalContext<'_>, text: Value<'_>) -> zbus::Result<()>;

    // ===== Stub Methods =====
    // These methods are required by the IBus interface but not used by XKey.
    // They are implemented as no-ops to satisfy the interface contract.

    /// Sets the capabilities of the client application.
    /// Not used by XKey.
    fn set_capabilities(&self, _caps: u32) -> zbus::fdo::Result<()> {
        Ok(())
    }

    /// Provides cursor location information for candidate window positioning.
    /// Not used by XKey (we don't show candidate windows).
    fn set_cursor_location(&self, _x: i32, _y: i32, _w: i32, _h: i32) -> zbus::fdo::Result<()> {
        Ok(())
    }

    /// Called when a property is activated (e.g., from language bar).
    /// Not used by XKey.
    fn property_activate(&self, _name: &str, _state: u32) -> zbus::fdo::Result<()> {
        Ok(())
    }

    /// Provides surrounding text context from the application.
    /// Not used by XKey.
    fn set_surrounding_text(
        &self,
        _text: &str,
        _cursor_pos: u32,
        _anchor_pos: u32,
    ) -> zbus::fdo::Result<()> {
        Ok(())
    }
}

/// IBus Factory implementation.
///
/// The factory is responsible for creating engine instances when IBus requests them.
/// In XKey, we use a single shared engine instance, so the factory simply returns
/// the path to our pre-registered engine.
pub struct XKeyFactory;

impl Default for XKeyFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl XKeyFactory {
    /// Creates a new factory instance.
    pub fn new() -> Self {
        Self
    }
}

/// D-Bus interface implementation for org.freedesktop.IBus.Factory
#[interface(name = "org.freedesktop.IBus.Factory")]
impl XKeyFactory {
    /// Creates an engine instance.
    ///
    /// Called by IBus when it needs an engine instance. Since we pre-register
    /// our engine, this simply returns the object path to the existing engine.
    ///
    /// # Arguments
    /// * `_engine_name` - Name of the engine to create (unused, we only have one)
    ///
    /// # Returns
    /// The D-Bus object path to the engine instance
    fn create_engine(
        &self,
        _engine_name: &str,
    ) -> zbus::fdo::Result<zvariant::ObjectPath<'static>> {
        Ok(zvariant::ObjectPath::try_from(OBJ_PATH).unwrap())
    }

    /// Called when the factory should be destroyed.
    /// Since we run as a standalone process, this is a no-op.
    fn destroy(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }
}

/// Type alias for IBus property maps (used in serialization).
type PropMap = HashMap<String, OwnedValue>;

/// Converts a Rust string into an IBusText D-Bus structure.
///
/// IBusText is the standard text representation in IBus, consisting of:
/// - Type name: "IBusText"
/// - Properties: Empty hashmap (no additional properties)
/// - Text content: The actual string
/// - Attributes: IBusAttrList for text formatting (empty in our case)
///
/// The structure follows the IBus D-Bus protocol format:
/// ```text
/// IBusText = ("IBusText", {properties}, text_string, IBusAttrList)
/// IBusAttrList = ("IBusAttrList", {properties}, [attributes])
/// ```
///
/// # Arguments
/// * `s` - The string to convert
///
/// # Returns
/// A D-Bus Value containing the IBusText structure
fn ibus_text(s: String) -> Value<'static> {
    // Create an empty IBusAttrList (no text attributes like underline or color)
    // Format: ("IBusAttrList", {empty_properties}, [empty_attributes])
    let attr_list_struct =
        Structure::from(("IBusAttrList", PropMap::new(), Vec::<OwnedValue>::new()));
    let attr_list_v: Value<'static> = Value::from(attr_list_struct);

    // Create the IBusText structure with the string and empty attributes
    // Format: ("IBusText", {empty_properties}, text_string, attr_list_variant)
    let text_struct = Structure::from((
        "IBusText",
        PropMap::new(),
        s,
        attr_list_v, // This Value will be encoded as a Variant inside the Structure
    ));

    Value::from(text_struct)
}
