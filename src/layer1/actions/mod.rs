//! Actions
//!
//! Defines the various actions pops can take, from simple tasks to complex behaviors like protests and mental breaks.

pub mod escape;
pub(crate) mod simple;
pub(crate) use simple::*;
pub(crate) mod clothing;
pub(crate) use clothing::*;
pub(crate) mod tool;
pub(crate) use tool::*;
pub(crate) mod research;
pub(crate) use research::*;
pub(crate) mod haul;
pub(crate) use haul::*;
pub(crate) mod hum;
pub(crate) use hum::*;
pub(crate) mod drafted;
pub(crate) use drafted::*;
pub(crate) mod mental_break;
pub(crate) use mental_break::*;
pub(crate) mod sabotage;
pub(crate) mod shower;

pub(crate) use shower::*;

pub(crate) mod clean;
pub(crate) use clean::*;

pub(crate) mod protest;
pub(crate) use protest::*;

pub(crate) mod gossip;
pub(crate) use gossip::*;

pub use crate::layer1::utility_types::AssignmentType;
use bevy_ecs::prelude::*;

/// Component tracking what a pop is assigned to.
#[derive(Component, Debug)]
pub struct AssignedTo {
    /// The entity the pop is assigned to.
    pub entity: Entity,
    /// The type of assignment.
    pub assignment_type: AssignmentType,
}
pub mod feral;
pub use feral::*;
