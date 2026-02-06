use crate::layer1::execution::{AssignedTo, AssignmentType};
use crate::layer1::resources::ColonyResources;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Available technologies in the tech tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tech {
    /// Allows construction of advanced stone buildings.
    Masonry,
    /// Allows working with metal (Smelter).
    MetalWorking,
    /// Allows advanced social buildings (Tavern).
    SocialStructures,
}

impl Tech {
    /// Returns the Knowledge cost to unlock this technology.
    #[must_use]
    pub const fn cost(&self) -> f32 {
        match self {
            Self::Masonry => 10.0,
            Self::MetalWorking => 20.0,
            Self::SocialStructures => 15.0,
        }
    }

    /// Returns the human-readable name of the technology.
    #[must_use]
    pub const fn label(&self) -> &str {
        match self {
            Self::Masonry => "Masonry",
            Self::MetalWorking => "Metal Working",
            Self::SocialStructures => "Social Structures",
        }
    }
}

/// Resource tracking the state of technology research.
#[derive(Resource, Default, Debug)]
pub struct TechState {
    /// The set of unlocked technologies.
    pub unlocked: HashSet<Tech>,
}

impl TechState {
    /// Checks if a specific technology is unlocked.
    #[must_use]
    pub fn is_unlocked(&self, tech: Tech) -> bool {
        self.unlocked.contains(&tech)
    }

    /// Unlocks a technology.
    pub fn unlock(&mut self, tech: Tech) {
        self.unlocked.insert(tech);
    }
}

/// Component marker for Library buildings.
#[derive(Component, Default)]
pub struct Library;

/// Attempts to unlock a technology using Knowledge.
///
/// Returns `true` if successful (affordable and not already unlocked, or already unlocked).
/// Deducts Knowledge from `ColonyResources`.
pub fn unlock_tech(world: &mut World, tech: Tech) -> bool {
    // Check if already unlocked?
    if world.resource::<TechState>().is_unlocked(tech) {
        return true;
    }

    let cost = tech.cost();
    let can_afford = {
        let res = world.resource::<ColonyResources>();
        res.knowledge >= cost
    };

    if can_afford {
        world.resource_mut::<ColonyResources>().knowledge -= cost;
        world.resource_mut::<TechState>().unlock(tech);

        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add(format!("Researched: {}", tech.label()));
        }
        true
    } else {
        false
    }
}

/// Generates knowledge based on pops working at Libraries.
pub fn process_research_system(world: &mut World) {
    // 1. Identify active libraries and count workers
    let mut library_workers = std::collections::HashMap::<Entity, u32>::new();

    let mut pops = world.query::<&AssignedTo>();
    for assignment in pops.iter(world) {
        if assignment.assignment_type == AssignmentType::LibraryWorker {
            *library_workers.entry(assignment.entity).or_insert(0) += 1;
        }
    }

    // 2. Iterate libraries and produce knowledge
    let mut total_knowledge_gained = 0.0;

    let mut libraries = world.query_filtered::<Entity, With<Library>>();
    for library_entity in libraries.iter(world) {
        if let Some(&workers) = library_workers.get(&library_entity) {
            // 0.01 knowledge per worker per tick
            // 100 ticks = 1 knowledge.
            // With 10 FPS, 10 seconds = 1 knowledge.
            // Cost 10 = 100 seconds per worker. Reasonable.
            #[allow(clippy::cast_precision_loss)]
            let knowledge_gain = 0.01 * workers as f32;
            total_knowledge_gained += knowledge_gain;
        }
    }

    if total_knowledge_gained > 0.0 {
        let mut res = world.resource_mut::<ColonyResources>();
        res.knowledge = (res.knowledge + total_knowledge_gained).clamp(0.0, res.max_knowledge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{BuildingType, try_place_building};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_resources_knowledge_fields() {
        let res = ColonyResources::default();
        assert_eq!(res.knowledge, 0.0);
        assert_eq!(res.max_knowledge, 100.0);
    }

    #[test]
    fn test_tech_enum_exists() {
        // Just verifying variants exist
        let _ = Tech::Masonry;
        let _ = Tech::MetalWorking;
        let _ = Tech::SocialStructures;
    }

    #[test]
    fn test_tech_costs() {
        assert_eq!(Tech::Masonry.cost(), 10.0);
        assert_eq!(Tech::MetalWorking.cost(), 20.0);
    }

    #[test]
    fn test_tech_state_starts_empty() {
        let state = TechState::default();
        assert!(!state.is_unlocked(Tech::Masonry));
        assert!(!state.is_unlocked(Tech::MetalWorking));
    }

    #[test]
    fn test_unlock_tech_success() {
        let mut world = World::new();
        world.insert_resource(TechState::default());
        world.insert_resource(MessageLog::default());
        let mut res = ColonyResources::default();
        res.knowledge = 20.0;
        world.insert_resource(res);

        let success = unlock_tech(&mut world, Tech::Masonry);

        assert!(success);
        assert!(world.resource::<TechState>().is_unlocked(Tech::Masonry));

        let res = world.resource::<ColonyResources>();
        assert!((res.knowledge - 10.0).abs() < f32::EPSILON); // Cost 10
    }

    #[test]
    fn test_unlock_tech_insufficient_funds() {
        let mut world = World::new();
        world.insert_resource(TechState::default());
        let mut res = ColonyResources::default();
        res.knowledge = 5.0; // Need 10
        world.insert_resource(res);

        let success = unlock_tech(&mut world, Tech::Masonry);

        assert!(!success);
        assert!(!world.resource::<TechState>().is_unlocked(Tech::Masonry));
        assert_eq!(world.resource::<ColonyResources>().knowledge, 5.0);
    }

    #[test]
    fn test_building_requires_tech() {
        assert_eq!(BuildingType::Housing.required_tech(), None); // Basic
        assert_eq!(
            BuildingType::Smelter.required_tech(),
            Some(Tech::MetalWorking)
        );
        assert_eq!(
            BuildingType::Tavern.required_tech(),
            Some(Tech::SocialStructures)
        );
    }

    #[test]
    fn test_placement_fails_if_tech_locked() {
        let mut world = World::new();
        world.insert_resource(TechState::default());
        world.insert_resource(MessageLog::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ..Default::default()
        }); // Plenty of resources

        // Setup Grid
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        // Try to place Smelter (needs MetalWorking)
        let success = try_place_building(&mut world, 5, 5, BuildingType::Smelter);

        assert!(!success);
    }

    #[test]
    fn test_placement_succeeds_if_tech_unlocked() {
        let mut world = World::new();
        let mut state = TechState::default();
        state.unlock(Tech::MetalWorking);
        world.insert_resource(state);
        world.insert_resource(MessageLog::default());

        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ore: 10.0,
            ..Default::default()
        }); // Resources + Cost checks

        // Setup Grid
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        // Try to place Smelter
        let success = try_place_building(&mut world, 5, 5, BuildingType::Smelter);

        assert!(success);
    }
}
