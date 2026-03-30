use crate::layer1::{GridPosition, MaterialType};
use bevy_ecs::prelude::{Component, Entity, Resource};
use std::collections::{HashMap, HashSet};
use crate::layer1::building::types::BuildingType;


/// Building component - attached to building entities.
#[derive(Component, Default)]
pub struct Building {
    /// The type of this building.
    pub building_type: BuildingType,
}

/// Build mode state resource.
#[derive(Resource, Default)]
pub struct BuildMode {
    /// Whether build mode is currently active.
    pub active: bool,
    /// The current cursor position in grid coordinates.
    pub cursor: GridPosition,
    /// The currently selected building type.
    pub selected: BuildingType,
    /// The currently selected material.
    pub selected_material: MaterialType,
}

/// Tracks which tiles have buildings (for placement validation).
#[derive(Resource, Default)]
pub struct OccupiedTiles(pub HashSet<(i32, i32)>);

/// A spatial map of buildings for fast lookup (Pos -> Entity).
#[derive(Resource, Default)]
pub struct BuildingMap(pub HashMap<(i32, i32), Entity>);

/// Defines when a building operates.
#[derive(Component, Debug, Clone, Copy)]
pub struct ShiftSchedule {
    /// Whether the building operates during the day (Dawn, Day, Dusk).
    pub day_shift: bool,
    /// Whether the building operates during the night.
    pub night_shift: bool,
}

impl Default for ShiftSchedule {
    fn default() -> Self {
        Self {
            day_shift: true,
            night_shift: false,
        }
    }
}

impl ShiftSchedule {
    /// Checks if the schedule is active for the given time of day.
    #[must_use]
    pub const fn is_active(&self, time: crate::layer1::day_night::TimeOfDay) -> bool {
        use crate::layer1::day_night::TimeOfDay;
        match time {
            TimeOfDay::Night => self.night_shift,
            _ => self.day_shift,
        }
    }
}

/// Building component indicating it was constructed in a vacuum.
///
/// Vacuum Welded buildings:
/// - Have +100% Max HP.
/// - Cannot be Repaired or Demolished.
/// - Must be Destroyed (yielding 0 resources).
#[derive(Component, Default)]
pub struct VacuumWelded;