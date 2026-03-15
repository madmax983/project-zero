#![allow(clippy::float_cmp, clippy::uninlined_format_args)]
//! Lighting system for the colony.
//!
//! Handles:
//! - Light levels on the grid.
//! - Ambient light (sun/moon).
//! - Light sources (lamps, fire).
//! - Effects on pop speed and morale.

use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Speed;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// Global ambient light level (0.0 = pitch black, 1.0 = bright day).
#[derive(Resource)]
pub struct AmbientLight {
    /// The global light intensity (0.0 to 1.0).
    pub level: f32,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self { level: 1.0 }
    }
}

/// Grid storing light levels for each tile.
#[derive(Resource)]
pub struct LightMap {
    /// Width of the map in tiles.
    pub width: u32,
    /// Height of the map in tiles.
    pub height: u32,
    /// Linear vector of light levels (row-major).
    pub tiles: Vec<f32>,
}

impl LightMap {
    /// Create a new light map with given dimensions.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            tiles: vec![0.0; (width * height) as usize],
        }
    }

    /// Get light level at (x, y). Returns 0.0 if out of bounds.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        self.tiles[(y * self.width + x) as usize]
    }

    /// Set light level at (x, y). Clamps value between 0.0 and 1.0.
    pub fn set(&mut self, x: u32, y: u32, val: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.tiles[(y * self.width + x) as usize] = val.clamp(0.0, 1.0);
    }
}

/// Component for entities that emit light.
#[derive(Component)]
pub struct LightSource {
    /// Maximum radius of the light in tiles.
    pub radius: f32,
    /// Intensity at the source center (0.0 to 1.0+).
    pub intensity: f32,
    /// Color of the light (RGB). Currently unused for logic, visual only.
    pub color: (u8, u8, u8),
    pub is_outdoor: bool,
}

impl Default for LightSource {
    fn default() -> Self {
        Self {
            radius: 0.0,
            intensity: 0.0,
            color: (255, 255, 255),
            is_outdoor: false,
        }
    }
}

/// Updates the light map based on ambient light and active light sources.
pub fn update_lighting_system(
    mut light_map: ResMut<LightMap>,
    ambient: Res<AmbientLight>,
    sources: Query<(&LightSource, &GridPosition, Option<&PowerConsumer>)>,
) {
    // 1. Reset map to Ambient
    light_map.tiles.fill(ambient.level);

    // 2. Iterate sources and spread light
    for (source, pos, power) in &sources {
        if power.is_some_and(|p| !p.active) {
            continue;
        }

        #[allow(clippy::cast_possible_truncation)]
        let radius_ceil = source.radius.ceil() as i32;
        let center_x = pos.x;
        let center_y = pos.y;

        for dy in -radius_ceil..=radius_ceil {
            for dx in -radius_ceil..=radius_ceil {
                #[allow(clippy::cast_precision_loss)]
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq > source.radius * source.radius {
                    continue;
                }

                let dist = dist_sq.sqrt();
                let x = center_x + dx;
                let y = center_y + dy;

                #[allow(clippy::cast_possible_wrap)]
                if x < 0 || y < 0 || x >= light_map.width as i32 || y >= light_map.height as i32 {
                    continue;
                }

                // Simple linear falloff: 1.0 at center, 0.0 at radius
                let falloff = (1.0 - (dist / source.radius)).max(0.0);
                let intensity = source.intensity * falloff;

                #[allow(clippy::cast_sign_loss)]
                let current = light_map.get(x as u32, y as u32);
                #[allow(clippy::cast_sign_loss)]
                light_map.set(x as u32, y as u32, current.max(intensity));
            }
        }
    }
}

/// Applies penalties (speed, morale) to pops in darkness.
pub fn apply_lighting_penalties_system(
    light_map: Res<LightMap>,
    mut pops: Query<(&GridPosition, &mut Speed, &mut Needs, Option<&Traits>)>,
) {
    for (pos, mut speed, mut needs, traits) in &mut pops {
        // Safe cast: GridPosition shouldn't be negative in valid map area
        let x = u32::try_from(pos.x).unwrap_or(0);
        let y = u32::try_from(pos.y).unwrap_or(0);

        let light = light_map.get(x, y);

        if light < 0.1 {
            // Darkness penalty
            speed.current *= 0.5;

            // Morale penalty (reduce leisure)
            let mut stress_factor = 1.0;

            if let Some(t) = traits {
                if t.0.contains(&Trait::Anxious) {
                    stress_factor = 2.0; // Panic!
                } else if t.0.contains(&Trait::NightOwl) {
                    stress_factor = 0.1; // Minimal stress
                }
            }

            let penalty = 0.005 * stress_factor;
            needs.leisure = (needs.leisure - penalty).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_light_map_initialization() {
        let map = LightMap::new(10, 10);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        assert_eq!(map.tiles.len(), 100);
        // Default should be 0.0
        assert!(map.get(0, 0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ambient_light_resource() {
        let ambient = AmbientLight::default();
        assert_eq!(ambient.level, 1.0); // Default to full brightness (Day)
    }

    #[test]
    fn test_light_source_component() {
        let source = LightSource {
            radius: 5.0,
            intensity: 1.0,
            color: (255, 255, 255),
            is_outdoor: false,
        };
        assert_eq!(source.radius, 5.0);
    }

    #[test]
    fn test_update_lighting_system_ambient() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        world.insert_resource(AmbientLight { level: 0.5 });

        // No sources
        world.run_system_once(update_lighting_system).unwrap();

        let map = world.resource::<LightMap>();
        // All tiles should be at least ambient level
        for tile in &map.tiles {
            assert!(
                (tile - 0.5).abs() < f32::EPSILON,
                "Tile level {} != 0.5",
                tile
            );
        }
    }

    #[test]
    fn test_update_lighting_system_source() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        world.insert_resource(AmbientLight { level: 0.0 }); // Pitch black

        // Add a light source at (5, 5)
        world.spawn((
            LightSource {
                radius: 2.0,
                intensity: 1.0,
                color: (255, 255, 255),
                is_outdoor: false,
            },
            GridPosition { x: 5, y: 5 },
        ));

        world.run_system_once(update_lighting_system).unwrap();

        let map = world.resource::<LightMap>();

        // Center should be bright (1.0)
        assert!(
            map.get(5, 5) >= 0.9,
            "Center should be bright, got {}",
            map.get(5, 5)
        );

        // Neighbor (6, 5) -> dist=1.0. 1.0 - (1.0/2.0) = 0.5.
        assert!(
            map.get(6, 5) > 0.1,
            "Neighbor should be lit, got {}",
            map.get(6, 5)
        );

        // Far away should be black
        assert!(
            map.get(0, 0) < 0.1,
            "Far away should be dark, got {}",
            map.get(0, 0)
        );
    }

    #[test]
    fn test_darkness_affects_speed() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        // Pitch black
        {
            let mut map = world.resource_mut::<LightMap>();
            map.tiles.fill(0.0);
        }

        // Pop in darkness
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Speed::default(),
                Needs::default(),
            ))
            .id();

        // Run system that updates speed based on light
        world
            .run_system_once(apply_lighting_penalties_system)
            .unwrap();

        let speed = world.get::<Speed>(pop).unwrap();
        // Should be penalized (e.g. 0.5)
        assert!(
            speed.current < speed.base,
            "Speed current {} should be < base {}",
            speed.current,
            speed.base
        );
    }

    #[test]
    fn test_light_affects_morale() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        // Pitch black
        {
            let mut map = world.resource_mut::<LightMap>();
            map.tiles.fill(0.0);
        }

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Speed::default(),
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        // Run Reset + Lighting
        world
            .run_system_once(crate::layer1::pop::reset_speed_system)
            .unwrap();
        world
            .run_system_once(apply_lighting_penalties_system)
            .unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 1.0, "Darkness should reduce leisure");
    }

    #[test]
    fn test_light_restores_speed() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        // Full brightness
        {
            let mut map = world.resource_mut::<LightMap>();
            map.tiles.fill(1.0);
        }

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Speed {
                    base: 1.0,
                    current: 0.5, // Previously penalized
                    accumulator: 0.0,
                },
                Needs::default(),
            ))
            .id();

        // Run Reset + Lighting
        world
            .run_system_once(crate::layer1::pop::reset_speed_system)
            .unwrap();
        world
            .run_system_once(apply_lighting_penalties_system)
            .unwrap();

        let speed = world.get::<Speed>(pop).unwrap();
        assert!(
            (speed.current - 1.0).abs() < f32::EPSILON,
            "Speed should be restored to base, got {}",
            speed.current
        );
    }
}
