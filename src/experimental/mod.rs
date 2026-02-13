//! Experimental features.
//!
//! Features in incubation by Nova.

/// Experimental acoustics feature (Weather/Industrial).
#[cfg(feature = "nova")]
pub mod acoustics;
/// Pop biographies.
pub mod biography;
/// Pop dreams.
pub mod dreams;
/// Experimental ghost feature.
#[cfg(feature = "nova")]
pub mod ghosts;
/// Experimental miasma feature.
#[cfg(feature = "nova")]
pub mod miasma;
/// Seasonal visual overlays.
pub mod seasonal_gfx;
