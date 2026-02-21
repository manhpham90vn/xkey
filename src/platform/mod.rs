//! Platform-specific input method implementations.
//!
//! This module provides platform-specific backends:
//! - **Linux**: IBus engine via D-Bus (zbus)
//! - **macOS**: InputMethodKit (IMKServer/IMKInputController)
//! - **Windows**: Low-level keyboard hook + SendInput

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;
