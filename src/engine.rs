use crate::core::{Action, CoreState, handle_key};
use std::sync::{Arc, Mutex};
use zbus::{interface, object_server::SignalContext};

/// The D-Bus object path where the IBus engine is exposed.
pub const OBJ_PATH: &str = "/org/freedesktop/IBus/Engine/xkey";

/// Holds the internal state of the engine.
#[derive(Default)]
pub struct EngineState {
    /// The core input logic state, including the character buffer.
    pub core: CoreState,
}

/// The main IBus Engine structure.
/// It wraps the `EngineState` in a thread-safe `Arc<Mutex<...>>` to allow
/// concurrent access from D-Bus event handlers.
pub struct XKey {
    pub st: Arc<Mutex<EngineState>>,
}

impl XKey {
    /// Creates a new instance of the XKey engine with default state.
    pub fn new() -> Self {
        Self {
            st: Arc::new(Mutex::new(EngineState::default())),
        }
    }

    /// Iterates through a list of actions and applies them by sending D-Bus signals.
    /// Actions include updating preedit text, committing text, or hiding preedit.
    async fn apply_actions(
        &self,
        ctxt: &SignalContext<'_>,
        actions: Vec<Action>,
    ) -> zbus::fdo::Result<bool> {
        let mut consume = false;

        for a in actions {
            match a {
                Action::UpdatePreedit {
                    text,
                    caret,
                    visible,
                } => {
                    // Signal IBus to update the preedit text (underlined text while typing)
                    Self::update_preedit_text(ctxt, text, caret as u32, visible).await?;
                }
                Action::HidePreedit => {
                    // Signal IBus to hide the preedit area
                    Self::hide_preedit_text(ctxt).await?;
                }
                Action::Commit(s) => {
                    // Signal IBus to commit final text to the application
                    Self::commit_text(ctxt, s).await?;
                }
                Action::Consume => consume = true, // Tell IBus that we handled this key event
                Action::PassThrough => {}          // Let the key pass through to the application
            }
        }
        Ok(consume)
    }
}

/// Implementation of the `org.freedesktop.IBus.Engine` D-Bus interface.
/// This trait defines the standard methods that IBus calls to interact with the engine.
#[interface(name = "org.freedesktop.IBus.Engine")]
impl XKey {
    /// Called by IBus whenever a key is pressed or released.
    async fn process_key_event(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        keyval: u32,
        _keycode: u32,
        state: u32,
    ) -> zbus::fdo::Result<bool> {
        // We lock the state to handle the key event and get a list of actions to perform.
        let actions = {
            let mut guard = self.st.lock().unwrap();
            handle_key(&mut guard.core, keyval, state)
        };
        // Apply the resulting actions (UpdatePreedit, Commit, etc.)
        self.apply_actions(&ctxt, actions).await
    }

    /// Called when the input focus leaves the current application window.
    async fn focus_out(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        {
            let mut guard = self.st.lock().unwrap();
            guard.core.buffer.clear(); // Clear internal buffer to avoid carrying over text
        }
        Self::hide_preedit_text(&ctxt).await?;
        Ok(())
    }

    /// Called when the engine is reset (e.g., via IBus menu).
    async fn reset(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        {
            let mut guard = self.st.lock().unwrap();
            guard.core.buffer.clear();
        }
        Self::hide_preedit_text(&ctxt).await?;
        Ok(())
    }

    // ===== Signals =====
    // These signals are defined by the IBus Engine interface to communicate back to IBus.

    /// Updates the text shown in the preedit area.
    #[zbus(signal)]
    pub async fn update_preedit_text(
        ctxt: &SignalContext<'_>,
        text: String,
        cursor_pos: u32,
        visible: bool,
    ) -> zbus::Result<()>;

    /// Hides the preedit area.
    #[zbus(signal)]
    pub async fn hide_preedit_text(ctxt: &SignalContext<'_>) -> zbus::Result<()>;

    /// Commits the specified text to the application.
    #[zbus(signal)]
    pub async fn commit_text(ctxt: &SignalContext<'_>, text: String) -> zbus::Result<()>;

    // Required by trait but not currently implemented for advanced features
    fn set_capabilities(&self, _caps: u32) -> zbus::fdo::Result<()> {
        Ok(())
    }
    fn set_cursor_location(&self, _x: i32, _y: i32, _w: i32, _h: i32) -> zbus::fdo::Result<()> {
        Ok(())
    }
    fn property_activate(&self, _name: &str, _state: u32) -> zbus::fdo::Result<()> {
        Ok(())
    }
    fn set_surrounding_text(
        &self,
        _text: &str,
        _cursor_pos: u32,
        _anchor_pos: u32,
    ) -> zbus::fdo::Result<()> {
        Ok(())
    }
}
