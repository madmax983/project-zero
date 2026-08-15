//! Stub implementation for Oral Tradition when the `nova` feature is not enabled.

use bevy_ecs::prelude::*;

/// A legend that has evolved from a historical event.
///
/// > **Note:** The `nova` feature is not enabled. This is a stub implementation.
#[derive(Debug, Clone, Default)]
pub struct Story {
    pub text: String,
    pub historical_date: u64,
    pub mutations: u32,
    pub genre: StoryGenre,
}

/// The thematic genre of a [`Story`].
///
/// > **Note:** The `nova` feature is not enabled. This is a stub implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StoryGenre {
    #[default]
    Trivial,
    Heroic,
    Tragedy,
    Cautionary,
}

impl std::fmt::Display for StoryGenre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Trivial")
    }
}

/// The collective repository of legends, myths, and rumors known to the colony.
///
/// > **Note:** The `nova` feature is not enabled. This is a stub implementation.
#[derive(Debug, Resource)]
pub struct OralTradition {
    pub stories: Vec<Story>,
}

impl Default for OralTradition {
    fn default() -> Self {
        eprintln!("⚠️ WARNING: `OralTradition` initialized but the `nova` feature is not enabled.");
        eprintln!("⚠️ The storytelling mechanics will not be active. Enable `features = [\"nova\"]` in your Cargo.toml.");
        Self {
            stories: Vec::new(),
        }
    }
}

impl OralTradition {
    pub fn add_story(&mut self, story: Story) {
        self.stories.push(story);
    }
}

impl std::fmt::Display for OralTradition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Oral Tradition (Stub - Enable `nova` feature)")
    }
}
