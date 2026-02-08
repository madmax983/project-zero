use bevy_ecs::prelude::*;

/// Hunger satisfaction action logic.
pub mod hunger;
/// Rest satisfaction action logic.
pub mod rest;

/// Work action logic.
pub mod work;
/// Repair action logic.
pub mod repair;
/// Research action logic.
pub mod research;
/// Haul action logic.
pub mod haul;
/// Explore action logic.
pub mod explore;
/// Idle action logic.
pub mod idle;

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
