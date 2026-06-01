//! The SCALE Prelude.
//!
//! This module provides easy access to commonly used types and functions in the SCALE library.
//! By adding `use scale::prelude::*;` to your code, you can avoid deep nested imports.

pub use crate::setup::{setup_world_with_config, SetupConfig};
pub use crate::shared::narrative::{NarrativeContext, NarrativeGenerator, NarrativeSegment};
pub use crate::shared::time::SimulationTime;
pub use crate::simulation::run_simulation_tick;

pub use crate::layer1::core::chronicle::{Chronicle, EventImportance};
pub use crate::layer1::psychology::needs::Needs;
pub use crate::layer1::social::Tavern;
pub use crate::shared::log::MessageLog;

#[cfg(feature = "nova")]
pub use crate::layer1::oral_tradition::{
    collect_chronicles_system, storytelling_system, OralTradition, Story, StoryGenre,
};

// --- Missing Feature Stubs for DX ---
#[cfg(not(feature = "nova"))]
#[deprecated(note = "🚨 ⚠️ REQUIRES FEATURE NOVA ⚠️ 🚨\nTo use this feature, you MUST enable the `nova` feature flag in your `Cargo.toml`.")]
#[allow(deprecated)]
pub struct OralTradition {
    pub stories: Vec<Story>,
}

#[cfg(not(feature = "nova"))]
#[allow(deprecated)]
impl Default for OralTradition {
    fn default() -> Self {
        Self {
            stories: Vec::new(),
        }
    }
}

#[cfg(not(feature = "nova"))]
#[allow(deprecated)]
impl OralTradition {
    pub fn add_story(&mut self, _story: Story) {}
    pub fn process_chronicles(&mut self, _chronicle: &Chronicle) {}
}

#[cfg(not(feature = "nova"))]
#[deprecated(note = "🚨 ⚠️ REQUIRES FEATURE NOVA ⚠️ 🚨\nTo use this feature, you MUST enable the `nova` feature flag in your `Cargo.toml`.")]
#[allow(deprecated)]
pub struct Story {
    pub text: String,
    pub historical_date: u64,
    pub mutations: u32,
    pub genre: StoryGenre,
}

#[cfg(not(feature = "nova"))]
#[deprecated(note = "🚨 ⚠️ REQUIRES FEATURE NOVA ⚠️ 🚨\nTo use this feature, you MUST enable the `nova` feature flag in your `Cargo.toml`.")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryGenre {
    Heroic,
    Tragedy,
    Cautionary,
    Trivial,
}
