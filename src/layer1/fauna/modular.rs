use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Type of body part for modular fauna.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Default)]
pub enum BodyPartType {
    /// Head part (determines attack, sensory).
    #[default]
    Head,
    /// Torso part (determines health, size).
    Body,
    /// Limbs (determines speed).
    Limbs,
    /// Tail (balance, auxiliary attack).
    Tail,
    /// Skin/Fur/Scales (defense).
    Integument,
}

/// Aggregated statistics for a fauna entity.
#[derive(Debug, Clone, Default)]
pub struct FaunaStats {
    /// Maximum health points.
    pub health_max: f32,
    /// Attack damage.
    pub attack: f32,
    /// Movement speed multiplier.
    pub speed: f32,
    /// Damage reduction.
    pub defense: f32,
}

impl std::ops::Add for FaunaStats {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            health_max: self.health_max + other.health_max,
            attack: self.attack + other.attack,
            speed: self.speed + other.speed,
            defense: self.defense + other.defense,
        }
    }
}

/// A specific part of a fauna entity.
#[derive(Debug, Clone, Default)]
pub struct FaunaPart {
    /// The slot this part occupies.
    pub part_type: BodyPartType,
    /// Display name of the part.
    pub name: String,
    /// Stats contributed by this part.
    pub stats: FaunaStats,
    /// Resource produced when butchered or harvested (e.g., "Milk", "Venom").
    pub resource_drop: Option<String>,
}

/// Component representing the physical body of a fauna entity.
#[derive(Component, Debug, Default, Clone)]
pub struct FaunaBody {
    /// Name of the creature.
    pub name: String,
    /// Map of body parts.
    pub parts: HashMap<BodyPartType, FaunaPart>,
}

impl FaunaBody {
    /// Adds a part to the body.
    pub fn add_part(&mut self, part: FaunaPart) {
        self.parts.insert(part.part_type, part);
    }

    /// Calculates total stats from all parts.
    #[must_use]
    pub fn aggregate_stats(&self) -> FaunaStats {
        self.parts
            .values()
            .fold(FaunaStats::default(), |acc, part| acc + part.stats.clone())
    }

    /// Checks if the body can produce a specific resource.
    #[must_use]
    pub fn can_produce(&self, resource: &str) -> bool {
        self.parts
            .values()
            .any(|p| p.resource_drop.as_deref() == Some(resource))
    }
}
