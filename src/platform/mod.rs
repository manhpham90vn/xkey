//! Platform-specific input method implementations.
//!
//! This module provides platform-specific backends:
//! - **Linux**: IBus engine via D-Bus (zbus)
//! - **macOS**: InputMethodKit (IMKServer/IMKInputController)

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;
