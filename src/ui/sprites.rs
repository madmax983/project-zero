//! Character sprite definitions for UI elements.
//!
//! This module centralizes sprite logic (single chars/strings).

use crate::layer1::{BuildingType, DesignationType, Needs, TerrainType};

const HEALTHY_THRESHOLD: f32 = 0.6;
const WARNING_THRESHOLD: f32 = 0.3;

/// Returns the character representation of a terrain type.
pub const fn get_terrain_char(terrain: TerrainType) -> &'static str {
    match terrain {
        TerrainType::Grass => ".",
        TerrainType::Dirt => ",",
        TerrainType::Rock => "#",
        TerrainType::Water => "~",
    }
}

/// Returns the character representation of a building type.
pub const fn get_building_char(building: BuildingType) -> &'static str {
    match building {
        BuildingType::Housing => "⌂",
        BuildingType::Farm => "♣",
    }
}

/// Returns the character representation of a designation type.
pub const fn get_designation_char(designation: DesignationType) -> &'static str {
    match designation {
        DesignationType::Mine => "⛏",
        DesignationType::Demolish => "X",
    }
}

/// Returns the character representation for a pop based on their needs.
pub fn get_pop_char(needs: &Needs) -> &'static str {
    let health = needs.worst();
    if health > HEALTHY_THRESHOLD {
        "☺"
    } else if health > WARNING_THRESHOLD {
        "☻"
    } else {
        "☹"
    }
}
