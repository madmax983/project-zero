#![allow(clippy::collapsible_if)]
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::factions::{FactionMember, FactionState, Factions};
use crate::layer1::resources::ColonyResources;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Available technologies in the tech tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tech {
    /// Allows construction of advanced stone buildings.
    Masonry,
    /// Allows working with metal (Smelter).
    MetalWorking,
    /// Allows advanced social buildings (Tavern).
    SocialStructures,
    /// Allows construction of the Observatory.
    Astronomy,
    /// Allows construction of Hydroponics Bays.
    Hydroponics,
    /// Allows active defense (Trash Cannon, Militia).
    Militia,
}

impl Tech {
    /// Returns the Knowledge cost to unlock this technology.
    #[must_use]
    pub const fn cost(&self) -> f32 {
        match self {
            Self::Masonry => 10.0,
            Self::MetalWorking => 20.0,
            Self::SocialStructures => 15.0,
            Self::Astronomy => 50.0,
            Self::Hydroponics => 30.0,
            Self::Militia => 25.0,
        }
    }

    /// Returns the human-readable name of the technology.
    #[must_use]
    pub const fn label(&self) -> &str {
        match self {
            Self::Masonry => "Masonry",
            Self::MetalWorking => "Metal Working",
            Self::SocialStructures => "Social Structures",
            Self::Astronomy => "Astronomy",
            Self::Hydroponics => "Hydroponics",
            Self::Militia => "Militia",
        }
    }

    /// Returns the data storage cost (in TB) required to maintain this technology.
    #[must_use]
    pub const fn storage_cost(&self) -> f32 {
        match self {
            Self::Masonry => 5.0,
            Self::MetalWorking | Self::Militia => 10.0,
            Self::SocialStructures | Self::Hydroponics => 15.0,
            Self::Astronomy => 20.0,
        }
    }
}

/// Status of a researched technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechStatus {
    /// Technology is fully functional.
    Active,
    /// Technology is corrupted due to data loss/capacity issues.
    Corrupted,
}

/// Resource tracking the state of technology research.
#[derive(Resource, Default, Debug)]
pub struct TechState {
    /// The set of unlocked technologies and their status.
    pub techs: HashMap<Tech, TechStatus>,
    /// Total data capacity available (TB).
    pub total_capacity: f32,
    /// Currently used data capacity (TB).
    pub used_capacity: f32,
}

impl TechState {
    /// Checks if a specific technology is unlocked AND active.
    /// Returns false if the tech is not researched or is Corrupted.
    #[must_use]
    pub fn is_unlocked(&self, tech: Tech) -> bool {
        self.is_active(tech)
    }

    /// Checks if a specific technology has been researched (Active or Corrupted).
    #[must_use]
    pub fn is_researched(&self, tech: Tech) -> bool {
        self.techs.contains_key(&tech)
    }

    /// Checks if a specific technology is fully active.
    #[must_use]
    pub fn is_active(&self, tech: Tech) -> bool {
        self.techs.get(&tech) == Some(&TechStatus::Active)
    }

    /// Unlocks a technology (forces Active).
    /// Used by existing tests/code that bypass cost checks.
    pub fn unlock(&mut self, tech: Tech) {
        self.techs.insert(tech, TechStatus::Active);
        self.update_corruption(); // Recalculate used capacity
    }

    /// Attempts to unlock a technology if capacity allows.
    pub fn try_unlock(&mut self, tech: Tech) -> bool {
        if self.techs.contains_key(&tech) {
            return true;
        }

        if self.used_capacity + tech.storage_cost() > self.total_capacity {
            return false;
        }

        self.techs.insert(tech, TechStatus::Active);
        self.used_capacity += tech.storage_cost();
        true
    }

    /// Helper for tests to force a specific status.
    #[cfg(test)]
    pub fn force_unlock(&mut self, tech: Tech, status: TechStatus) {
        self.techs.insert(tech, status);
        self.update_corruption();
    }

    /// Returns the status of a tech, if unlocked.
    #[cfg(test)]
    pub fn status(&self, tech: Tech) -> TechStatus {
        *self.techs.get(&tech).unwrap_or(&TechStatus::Active)
    }

    /// Updates corruption state based on capacity.
    pub fn update_corruption(&mut self) {
        // Calculate used capacity from active techs
        let active_usage: f32 = self
            .techs
            .iter()
            .filter(|(_, status)| **status == TechStatus::Active)
            .map(|(t, _)| t.storage_cost())
            .sum();

        if active_usage > self.total_capacity {
            // Over capacity! Corrupt the MOST EXPENSIVE techs first.
            let mut active_techs: Vec<Tech> = self
                .techs
                .iter()
                .filter(|(_, s)| **s == TechStatus::Active)
                .map(|(t, _)| *t)
                .collect();

            active_techs.sort_by(|a, b| {
                b.storage_cost()
                    .partial_cmp(&a.storage_cost())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut current_usage = active_usage;
            for tech in active_techs {
                if current_usage <= self.total_capacity {
                    break;
                }

                self.techs.insert(tech, TechStatus::Corrupted);
                current_usage -= tech.storage_cost();
            }
        } else {
            // Auto-repair if capacity allows
            let mut corrupted_techs: Vec<Tech> = self
                .techs
                .iter()
                .filter(|(_, s)| **s == TechStatus::Corrupted)
                .map(|(t, _)| *t)
                .collect();

            // Sort by cost asc (restore cheap first)
            corrupted_techs.sort_by(|a, b| {
                a.storage_cost()
                    .partial_cmp(&b.storage_cost())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut current_usage = active_usage;
            for tech in corrupted_techs {
                if current_usage + tech.storage_cost() <= self.total_capacity {
                    self.techs.insert(tech, TechStatus::Active);
                    current_usage += tech.storage_cost();
                }
            }
        }

        // Final update of used_capacity
        self.used_capacity = self
            .techs
            .iter()
            .filter(|(_, status)| **status == TechStatus::Active)
            .map(|(t, _)| t.storage_cost())
            .sum();
    }
}

/// Component marker for Library buildings.
#[derive(Component, Default)]
pub struct Library;

/// Component providing data storage capacity.
#[derive(Component, Default)]
pub struct DataStorage {
    /// Capacity in Terabytes (TB).
    pub capacity: f32,
}

/// Attempts to unlock a technology using Knowledge.
///
/// Returns `true` if successful (affordable and not already unlocked, or already unlocked).
/// Deducts Knowledge from `ColonyResources`.
pub fn unlock_tech(world: &mut World, tech: Tech) -> bool {
    // Check if already researched (even if Corrupted)?
    if world.resource::<TechState>().is_researched(tech) {
        return true;
    }

    let cost = tech.cost();
    let can_afford_resources = {
        let res = world.resource::<ColonyResources>();
        res.knowledge >= cost
    };

    if !can_afford_resources {
        return false;
    }

    // Try to unlock (checking capacity)
    // Note: We need to borrow TechState mutably
    let unlocked = world.resource_mut::<TechState>().try_unlock(tech);

    if unlocked {
        world.resource_mut::<ColonyResources>().knowledge -= cost;
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add(format!("Researched: {}", tech.label()));
        }
        true
    } else {
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add(format!(
                "Failed to research {}: Insufficient Data Capacity",
                tech.label()
            ));
        }
        false
    }
}

/// Generates knowledge based on pops working at Libraries.
pub fn process_research_system(
    pops: Query<(&AssignedTo, Option<&FactionMember>)>,
    libraries: Query<Entity, With<Library>>,
    mut resources: ResMut<ColonyResources>,
    factions: Option<Res<Factions>>,
) {
    let mut library_workers = std::collections::HashMap::<Entity, u32>::new();

    for (assignment, member_opt) in &pops {
        if assignment.assignment_type == AssignmentType::LibraryWorker {
            if let Some(factions) = &factions {
                if let Some(member) = member_opt {
                    if let Some(fid) = member.faction_id {
                        if factions
                            .get(fid)
                            .is_some_and(|d| d.state == FactionState::Striking)
                        {
                            continue;
                        }
                    }
                }
            }

            *library_workers.entry(assignment.entity).or_insert(0) += 1;
        }
    }

    let mut total_knowledge_gained = 0.0;

    for library_entity in &libraries {
        if let Some(&workers) = library_workers.get(&library_entity) {
            #[allow(clippy::cast_precision_loss)]
            let knowledge_gain = 0.01 * workers as f32;
            total_knowledge_gained += knowledge_gain;
        }
    }

    if total_knowledge_gained > 0.0 {
        resources.knowledge =
            (resources.knowledge + total_knowledge_gained).clamp(0.0, resources.max_knowledge);
    }
}

/// Updates global tech capacity based on powered servers.
pub fn update_tech_capacity_system(
    mut tech_state: ResMut<TechState>,
    query: Query<(&DataStorage, Option<&crate::layer1::energy::PowerConsumer>)>,
) {
    let total_cap: f32 = query
        .iter()
        .filter(|(_, power)| power.map_or(true, |p| p.active))
        .map(|(storage, _)| storage.capacity)
        .sum();

    tech_state.total_capacity = total_cap;
    tech_state.update_corruption();
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
        assert!(res.knowledge.abs() < f32::EPSILON);
        assert!((res.max_knowledge - 100.0).abs() < f32::EPSILON);
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
        assert!((Tech::Masonry.cost() - 10.0).abs() < f32::EPSILON);
        assert!((Tech::MetalWorking.cost() - 20.0).abs() < f32::EPSILON);
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
        let mut state = TechState::default();
        state.total_capacity = 100.0; // Needs capacity now!
        world.insert_resource(state);
        world.insert_resource(MessageLog::default());
        world.insert_resource(ColonyResources {
            knowledge: 20.0,
            ..Default::default()
        });

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
        world.insert_resource(ColonyResources {
            knowledge: 5.0, // Need 10
            ..Default::default()
        });

        let success = unlock_tech(&mut world, Tech::Masonry);

        assert!(!success);
        assert!(!world.resource::<TechState>().is_unlocked(Tech::Masonry));
        assert!((world.resource::<ColonyResources>().knowledge - 5.0).abs() < f32::EPSILON);
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
        state.total_capacity = 100.0;
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
