//! Experimental features.
//!
//! Features in incubation by Nova.

/// Pop biographies.
pub mod biography;
/// Pop dreams.
pub mod dreams;
/// Experimental ghost feature.
#[cfg(feature = "nova")]
pub mod ghosts;
/// Seasonal visual overlays.
pub mod seasonal_gfx;
