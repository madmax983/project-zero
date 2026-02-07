#![allow(clippy::match_wildcard_for_single_variants)]

use crate::layer1::seasons::Season;
use crate::layer1::terrain::TerrainType;
use ratatui::style::Color;

/// Returns an optional color override for terrain based on the season.
///
/// If `None` is returned, the default terrain color should be used.
#[must_use]
pub const fn get_texture_override(terrain: TerrainType, season: Option<Season>) -> Option<Color> {
    match season {
        Some(Season::Winter) => match terrain {
            TerrainType::Grass => Some(Color::Rgb(220, 220, 225)), // Snowy white/grey
            TerrainType::Dirt => Some(Color::Rgb(200, 190, 180)),  // Frozen dirt
            TerrainType::Tree => Some(Color::Rgb(100, 100, 100)),  // Bare branches
            TerrainType::Water => Some(Color::Cyan),               // Icy blue
            _ => None,
        },
        Some(Season::Autumn) => match terrain {
            TerrainType::Grass => Some(Color::Rgb(180, 140, 50)), // Drying grass
            TerrainType::Tree => Some(Color::Rgb(200, 80, 20)),   // Fall foliage
            _ => None,
        },
        Some(Season::Summer) => match terrain {
            TerrainType::Grass => Some(Color::Rgb(50, 205, 50)), // LimeGreen (Lush)
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winter_colors() {
        assert_eq!(
            get_texture_override(TerrainType::Grass, Some(Season::Winter)),
            Some(Color::Rgb(220, 220, 225))
        );
        assert_eq!(
            get_texture_override(TerrainType::Water, Some(Season::Winter)),
            Some(Color::Cyan)
        );
        // Rock should not change
        assert_eq!(
            get_texture_override(TerrainType::Rock, Some(Season::Winter)),
            None
        );
    }

    #[test]
    fn test_autumn_colors() {
        assert_eq!(
            get_texture_override(TerrainType::Tree, Some(Season::Autumn)),
            Some(Color::Rgb(200, 80, 20))
        );
        // Water should not change in Autumn
        assert_eq!(
            get_texture_override(TerrainType::Water, Some(Season::Autumn)),
            None
        );
    }

    #[test]
    fn test_spring_defaults() {
        // Spring should return None (default colors)
        assert_eq!(
            get_texture_override(TerrainType::Grass, Some(Season::Spring)),
            None
        );
    }

    #[test]
    fn test_none_season_defaults() {
        assert_eq!(get_texture_override(TerrainType::Grass, None), None);
    }
}
