//! Platform abstraction layer.
//!
//! This module provides platform-agnostic input types that both native (crossterm)
//! and WASM (ratzilla) backends translate into, allowing shared game logic to
//! handle input without coupling to a specific backend.

/// Native (crossterm) platform adapter.
#[cfg(feature = "native")]
pub mod native;

/// WASM (ratzilla) platform adapter.
#[cfg(feature = "wasm")]
pub mod wasm;
