use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct SetupConfig {
    /// If true, skip GPU initialization (for headless environments).
    pub headless: bool,
    /// Which built-in start scenario to use for startup plumbing.
    pub scenario: StartScenarioId,
}

/// Coarse difficulty tags for curated start scenarios.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartScenarioDifficulty {
    Hard,
    Medium,
    Easy,
    Standard,
}

/// Stable built-in identifiers for curated startup scenarios.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum StartScenarioId {
    #[default]
    Classic,
    GroundSurvival,
    SocialDrama,
    Layer2Ready,
}

impl StartScenarioId {
    /// All built-in start scenarios in stable menu order.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            Self::Classic,
            Self::GroundSurvival,
            Self::SocialDrama,
            Self::Layer2Ready,
        ]
    }
}

/// Static metadata for a built-in start scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartScenarioDefinition {
    pub id: StartScenarioId,
    pub name: &'static str,
    pub difficulty: StartScenarioDifficulty,
}

/// The startup scenario selected for the active world.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveStartScenario {
    pub id: StartScenarioId,
    pub name: &'static str,
    pub difficulty: StartScenarioDifficulty,
}

/// The startup scenario state already baked into the current world.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppliedStartScenario {
    pub id: StartScenarioId,
}

#[must_use]
pub const fn start_scenario_definition(id: StartScenarioId) -> StartScenarioDefinition {
    match id {
        StartScenarioId::Classic => StartScenarioDefinition {
            id,
            name: "Classic",
            difficulty: StartScenarioDifficulty::Standard,
        },
        StartScenarioId::GroundSurvival => StartScenarioDefinition {
            id,
            name: "Ground Survival",
            difficulty: StartScenarioDifficulty::Hard,
        },
        StartScenarioId::SocialDrama => StartScenarioDefinition {
            id,
            name: "Social Drama",
            difficulty: StartScenarioDifficulty::Medium,
        },
        StartScenarioId::Layer2Ready => StartScenarioDefinition {
            id,
            name: "Layer 2 Ready",
            difficulty: StartScenarioDifficulty::Easy,
        },
    }
}
