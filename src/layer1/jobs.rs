//! Job and assignment definitions.
//!
//! This module contains types related to what a Pop is doing, either as a persistent job
//! or a temporary assignment.

use bevy_ecs::prelude::*;

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
    /// Working at an observatory.
    ObservatoryWorker,
    /// Mining job.
    Miner,
    /// Hauling job.
    Hauler,
    /// Construction job.
    Builder,
    /// Crafting job.
    Crafter,
    /// Guard job.
    Guard,
    /// Engineering job.
    Engineer,
    /// Medical doctor job.
    Doctor,
    /// Merchant job.
    Merchant,
    /// Scientist job.
    Scientist,
    /// Artist job.
    Artist,
    /// Governor job.
    Governor,
    /// Administrator job.
    Administrator,
    /// Undergoing surgery.
    Surgery,
}

/// Tracks a pop's persistent employment, even when temporarily reassigned (e.g. to hospital).
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// The building entity where the pop works.
    pub workplace: Entity,
    /// The type of job (e.g. `FarmWorker`, `LibraryWorker`).
    pub job_type: AssignmentType,
}
