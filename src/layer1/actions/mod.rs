use bevy_ecs::prelude::*;

/// Hunger satisfaction action logic.
pub mod hunger;
/// Rest satisfaction action logic.
pub mod rest;

/// Explore action logic.
pub mod explore;
/// Fetch tool action logic.
pub mod fetch_tool;
/// Haul action logic.
pub mod haul;
/// Idle action logic.
pub mod idle;
/// Repair action logic.
pub mod repair;
/// Research action logic.
pub mod research;
/// Work action logic.
pub mod work;

/// Refine action logic.
pub mod refine;

/// Farm action logic.
pub mod farm;

#[cfg(test)]
mod work_building_tests;

/// Component tracking what a pop is assigned to.
#[derive(Component, Debug)]
pub struct AssignedTo {
    /// The entity the pop is assigned to.
    pub entity: Entity,
    /// The type of assignment.
    pub assignment_type: AssignmentType,
}

/// Types of assignments a pop can have.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssignmentType {
    /// Working at a farm.
    FarmWorker,
    /// Residing in housing.
    HousingResident,
    /// Socializing at a tavern.
    TavernVisitor,
    /// Working at a library.
    LibraryWorker,
    /// Recovering in a hospital.
    Patient,
    /// Burying a corpse.
    Funeral,
}

/// Helper to assign a pop to an entity with a specific role.
pub fn assign_pop(
    commands: &mut Commands,
    pop_entity: Entity,
    target_entity: Entity,
    assignment_type: AssignmentType,
) {
    commands.entity(pop_entity).insert(AssignedTo {
        entity: target_entity,
        assignment_type,
    });
}
