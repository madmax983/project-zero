use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Tags applied to a Pop representing their cultural drift and identity.
#[derive(Component, Debug, Clone, Default)]
pub struct CulturalTag {
    tags: HashSet<String>,
}

impl CulturalTag {
    /// Create a new `CulturalTag` from a list of strings.
    pub fn new(initial_tags: Vec<String>) -> Self {
        Self {
            tags: initial_tags.into_iter().collect(),
        }
    }

    /// Check if the pop has a specific cultural tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// Add a cultural tag to the pop.
    pub fn add_tag(&mut self, tag: &str) {
        self.tags.insert(tag.to_string());
    }
}
