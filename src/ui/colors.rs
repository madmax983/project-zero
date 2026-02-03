//! Color definitions for UI elements.
//!
//! This module centralizes color logic, preventing simulation types from knowing about `ratatui`.

use crate::layer1::{BuildingType, DesignationType, Needs, TerrainType};
use crate::shared::log::LogColor;
use ratatui::style::Color;

const HEALTHY_THRESHOLD: f32 = 0.6;
const WARNING_THRESHOLD: f32 = 0.3;

/// Returns the color associated with a terrain type.
pub const fn get_terrain_color(terrain: TerrainType) -> Color {
    match terrain {
        TerrainType::Grass => Color::Green,
        TerrainType::Dirt => Color::Rgb(139, 90, 43),
        TerrainType::Rock => Color::DarkGray,
        TerrainType::Water => Color::Blue,
    }
}

/// Returns the color associated with a building type.
pub const fn get_building_color(building: BuildingType) -> Color {
    match building {
        BuildingType::Housing => Color::Rgb(139, 90, 43), // Brown
        BuildingType::Farm => Color::Rgb(218, 165, 32),   // Goldenrod
    }
}

/// Returns the color associated with a designation type.
pub const fn get_designation_color(_designation: DesignationType) -> Color {
    Color::Red
}

/// Returns the color for a pop based on their needs.
pub fn get_pop_color(needs: &Needs) -> Color {
    let health = needs.worst();
    if health > HEALTHY_THRESHOLD {
        Color::Yellow
    } else if health > WARNING_THRESHOLD {
        Color::Rgb(255, 165, 0)
    } else {
        Color::Red
    }
}

/// Returns the UI color for a log message color.
pub const fn get_log_color(color: LogColor) -> Color {
    match color {
        LogColor::White => Color::White,
        LogColor::Red => Color::Red,
        LogColor::Green => Color::Green,
        LogColor::Yellow => Color::Yellow,
        LogColor::Blue => Color::Blue,
    }
}
