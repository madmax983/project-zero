use bevy_ecs::prelude::*;

/// Controls the state of a door (Airlock, Gate).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DoorControl {
    /// The current state of the door.
    pub state: DoorState,
}

/// The state of a door.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DoorState {
    /// Normal behavior (opens for pathing, closed otherwise).
    #[default]
    Auto,
    /// Forced open (vents pressure).
    Open,
    /// Forced closed (blocks pathing & pressure).
    Locked,
}
