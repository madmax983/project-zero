//! Navigation
//!
//! Defines navigation systems, inertial mechanics, and stellar weather interactions.

pub mod inertial;
pub mod stellar_weather;

pub use inertial::*;
pub use stellar_weather::*;
