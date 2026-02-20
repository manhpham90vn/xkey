//! XKey Core Library
//!
//! Exposes the core engine and platform-specific FFI logic so
//! they can be compiled as a static library for native platforms (like macOS).

pub mod core;
pub mod platform;
pub mod repl;
pub mod telex;
pub mod utils;
