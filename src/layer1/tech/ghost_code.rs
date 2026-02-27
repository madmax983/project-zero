use bevy_ecs::prelude::*;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use crate::layer1::events::{BuildingCompletedEvent, BuildingRemovedEvent};

/// Component representing digital residue left behind after a building is deconstructed.
///
/// This invisible entity persists on the tile and can "infect" future buildings constructed
/// on the same spot with [`GhostCode`].
#[derive(Component, Debug, Clone)]
pub struct DataResidue {
    /// The type of the building that left this residue.
    pub source_type: BuildingType,
}

/// Component representing "glitched" or "haunted" behavior inherited from previous structures.
#[derive(Component, Debug, Default)]
pub struct GhostCode {
    /// The specific behaviors or quirks inherited.
    pub traits: Vec<GhostTrait>,
}

/// Specific behaviors inherited from ghost code.
#[derive(Debug, Clone, PartialEq)]
pub enum GhostTrait {
    /// The building consumes power even when idle, or more power than usual.
    PhantomPower,
    /// The building exhibits targeting behavior from a previous defensive structure.
    LegacyTargeting,
    /// The building has a corrupted protocol string (Flavor/Lore).
    GhostProtocol(String),
}

impl DataResidue {
    /// Determines the trait to apply based on the source building type.
    #[must_use]
    pub fn get_ghost_trait(&self) -> GhostTrait {
        match self.source_type {
            BuildingType::Tower | BuildingType::TrashCannon => GhostTrait::LegacyTargeting,
            BuildingType::Hospital | BuildingType::CryoPod => GhostTrait::GhostProtocol("Triage".into()),
            _ => GhostTrait::PhantomPower,
        }
    }
}

/// System that spawns [`DataResidue`] when a building is removed.
pub fn residue_system(
    mut commands: Commands,
    mut events: EventReader<BuildingRemovedEvent>,
) {
    for event in events.read() {
        commands.spawn((
            DataResidue { source_type: event.building_type },
            event.position,
            // Name it for debug
            // Name::new("Data Residue"),
        ));
    }
}

/// System that checks if a new building is placed on [`DataResidue`] and applies [`GhostCode`].
pub fn ghost_infection_system(
    mut commands: Commands,
    mut events: EventReader<BuildingCompletedEvent>,
    building_query: Query<&GridPosition, With<Building>>,
    residue_query: Query<(Entity, &DataResidue, &GridPosition)>,
) {
    for event in events.read() {
        if let Ok(build_pos) = building_query.get(event.entity) {
            // Check for residue at this position
            for (residue_entity, residue, residue_pos) in residue_query.iter() {
                if build_pos == residue_pos {
                    // Infect!
                    commands.entity(event.entity).insert(GhostCode {
                        traits: vec![residue.get_ghost_trait()],
                    });

                    // Consume residue to prevent infinite stacking.
                    commands.entity(residue_entity).despawn();
                }
            }
        }
    }
}

/// Helper function to manually purge residue (e.g., via a Purge job).
pub fn perform_purge(world: &mut World, residue_entity: Entity) {
    if world.get::<DataResidue>(residue_entity).is_some() {
        world.despawn(residue_entity);
    }
}
