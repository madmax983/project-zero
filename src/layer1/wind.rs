#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    missing_docs
)]
use bevy_ecs::prelude::*;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use std::collections::HashSet;

/// Simple 2D vector for wind calculations (avoids external dependencies).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize_or_zero(self) -> Self {
        let len = self.length();
        if len < f32::EPSILON {
            Self::ZERO
        } else {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

/// Grid storing wind vectors for each tile.
#[derive(Resource)]
pub struct WindGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Wind vectors for each tile.
    pub vectors: Vec<Vec2>,
}

impl WindGrid {
    /// Create a new wind grid.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            vectors: vec![Vec2::ZERO; width * height],
        }
    }

    /// Get the wind vector at the specified coordinates.
    #[must_use]
    pub fn get_wind(&self, x: i32, y: i32) -> Vec2 {
        if x < 0 || y < 0 {
            return Vec2::ZERO;
        }
        let ux = x as usize;
        let uy = y as usize;
        if ux >= self.width || uy >= self.height {
            return Vec2::ZERO;
        }
        self.vectors[uy * self.width + ux]
    }

    /// Set the wind vector at the specified coordinates.
    pub fn set_wind(&mut self, x: i32, y: i32, wind: Vec2) {
        if x < 0 || y < 0 {
            return;
        }
        let ux = x as usize;
        let uy = y as usize;
        if ux >= self.width || uy >= self.height {
            return;
        }
        self.vectors[uy * self.width + ux] = wind;
    }
}

/// Global wind parameters affecting the entire map.
#[derive(Resource)]
pub struct GlobalWind {
    /// Normalized direction of the global wind.
    pub direction: Vec2,
    /// Base speed of the global wind.
    pub speed: f32,
}

impl Default for GlobalWind {
    fn default() -> Self {
        Self {
            direction: Vec2::X, // Default East wind
            speed: 1.0,
        }
    }
}

/// System to update local wind patterns based on terrain and buildings.
pub fn update_wind_system(
    mut wind_grid: ResMut<WindGrid>,
    global_wind: Option<Res<GlobalWind>>,
    terrain_grid: Option<Res<TerrainGrid>>,
    building_query: Query<(&Building, &GridPosition)>,
) {
    // 1. Gather resources
    let global = if let Some(g) = global_wind {
        (g.direction, g.speed)
    } else {
        return;
    };

    let base_wind = global.0.normalize_or_zero() * global.1;

    // 2. Identify blockers
    let mut blockers = HashSet::new();

    // Terrain blockers
    if let Some(terrain) = terrain_grid {
        for y in 0..terrain.height {
            for x in 0..terrain.width {
                if matches!(terrain.get(x, y), Some(TerrainType::Rock)) {
                    blockers.insert((x as i32, y as i32));
                }
            }
        }
    }

    // Building blockers
    for (b, pos) in building_query.iter() {
        if b.building_type.blocks_wind() {
            blockers.insert((pos.x, pos.y));
        }
    }

    // 3. Update grid
    let width = wind_grid.width as i32;
    let height = wind_grid.height as i32;

    for y in 0..height {
        for x in 0..width {
            if blockers.contains(&(x, y)) {
                wind_grid.set_wind(x, y, Vec2::ZERO);
                continue;
            }

            let mut local_wind = base_wind;

            // 1. Lee/Shadow Check
            // Check immediate upwind neighbor
            // We use round() to snap to the nearest grid cell in the upwind direction.
            let upwind_offset_x = (-base_wind.normalize_or_zero().x).round() as i32;
            let upwind_offset_y = (-base_wind.normalize_or_zero().y).round() as i32;
            let upwind_pos = (x + upwind_offset_x, y + upwind_offset_y);

            if blockers.contains(&upwind_pos) {
                local_wind = local_wind * 0.2; // Shadow penalty
            } else {
                // 2. Canyon Check
                // Check neighbors perpendicular to wind
                // Rotate 90 degrees: (x, y) -> (-y, x)
                // Re-normalize for direction vector
                let dir = base_wind.normalize_or_zero();
                let perp_x = (-dir.y).round() as i32;
                let perp_y = (dir.x).round() as i32;

                let side1 = (x + perp_x, y + perp_y);
                let side2 = (x - perp_x, y - perp_y);

                if blockers.contains(&side1) && blockers.contains(&side2) {
                    local_wind = local_wind * 1.5; // Canyon boost
                }
            }

            wind_grid.set_wind(x, y, local_wind);
        }
    }
}

/// Calculate movement cost modifier based on wind.
///
/// * `wind`: The wind vector at the current tile.
/// * `move_dir`: The direction of movement (e.g., (1,0) for East).
///
/// Returns a multiplier for movement cost (e.g., 0.8 for tailwind, 1.5 for headwind).
#[must_use]
pub fn calculate_wind_movement_penalty(wind: Vec2, move_dir: Vec2) -> f32 {
    let speed = wind.length();
    if speed < 0.1 { return 1.0; }

    let wind_dir = if speed > f32::EPSILON {
        Vec2 { x: wind.x / speed, y: wind.y / speed }
    } else {
        Vec2::ZERO
    };

    let dot = wind_dir.dot(move_dir.normalize_or_zero());

    // Dot = 1.0 (Tailwind) -> Cost < 1.0
    // Dot = -1.0 (Headwind) -> Cost > 1.0

    // Spec:
    // "Dot = 1.0 (Tailwind) -> Cost 0.8"
    // "Dot = -1.0 (Headwind) -> Cost 1.5"

    if dot > 0.0 {
        // Tailwind: Reduce cost
        // Max reduction 40% at speed 2.0? Spec code:
        // 1.0 - (dot * 0.2 * speed.min(2.0))
        // If speed=1.0, dot=1.0 -> 1.0 - 0.2 = 0.8. Correct.
        // Use mul_add optimization: a * b + c
        (dot * 0.2).mul_add(-speed.min(2.0), 1.0)
    } else {
        // Headwind: Increase cost
        // Spec code:
        // 1.0 + (dot.abs() * 0.5 * speed.min(2.0))
        // If speed=1.0, dot=-1.0 -> 1.0 + 0.5 = 1.5. Correct.
        (dot.abs() * 0.5).mul_add(speed.min(2.0), 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_wind_grid_initialization() {
        let grid = WindGrid::new(10, 10);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(grid.get_wind(0, 0), Vec2::ZERO);
    }

    #[test]
    fn test_global_wind_propagation() {
        let mut world = World::new();
        let width = 5;
        let height = 5;

        world.insert_resource(WindGrid::new(width, height));
        world.insert_resource(GlobalWind { direction: Vec2::new(1.0, 0.0), speed: 1.0 }); // East wind
        world.insert_resource(TerrainGrid {
             width,
             height,
             tiles: vec![TerrainType::Grass; width * height],
        });

        // Run system using schedule or directly if signature matches (but it needs params now)
        // Since we changed to SystemParam, we can't call it directly with &mut World.
        // We must use `world.run_system_once`.
        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, update_wind_system).unwrap();

        let grid = world.resource::<WindGrid>();
        // Center tile should match global wind in open terrain
        let wind = grid.get_wind(2, 2);
        assert!((wind.x - 1.0).abs() < 0.01, "Wind X should be 1.0, got {}", wind.x);
        assert!((wind.y - 0.0).abs() < 0.01, "Wind Y should be 0.0, got {}", wind.y);
    }

    #[test]
    fn test_wind_blocked_by_wall() {
        let mut world = World::new();
        let width = 5;
        let height = 5;
        world.insert_resource(WindGrid::new(width, height));
        world.insert_resource(GlobalWind { direction: Vec2::new(1.0, 0.0), speed: 1.0 }); // East wind

        // Spawn Wall at (1, 2)
        world.spawn((
            Building { building_type: BuildingType::Wall, ..Default::default() },
            GridPosition { x: 1, y: 2 },
        ));

        // Setup terrain (needed for checking blocks)
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; width * height],
        });

        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, update_wind_system).unwrap();

        let grid = world.resource::<WindGrid>();

        // Tile (2, 2) is directly downwind (East) of the wall at (1, 2).
        // It should be in the "Wind Shadow".
        let wind_shadow = grid.get_wind(2, 2);
        assert!(wind_shadow.length() < 0.5, "Wind should be reduced in lee of wall. Got length {}", wind_shadow.length());
    }

    #[test]
    fn test_urban_canyon_effect() {
        let mut world = World::new();
        let width = 5;
        let height = 5;
        world.insert_resource(WindGrid::new(width, height));
        world.insert_resource(GlobalWind { direction: Vec2::new(1.0, 0.0), speed: 1.0 }); // East wind
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; width * height],
        });

        // Create Canyon: Walls at y=1 and y=3. Wind flows along y=2.
        // Wall at (2, 1)
        world.spawn((
            Building { building_type: BuildingType::Wall, ..Default::default() },
            GridPosition { x: 2, y: 1 },
        ));
        // Wall at (2, 3)
        world.spawn((
            Building { building_type: BuildingType::Wall, ..Default::default() },
            GridPosition { x: 2, y: 3 },
        ));

        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, update_wind_system).unwrap();

        let grid = world.resource::<WindGrid>();

        // Tile (2, 2) is in the canyon. Wind should be accelerated.
        let canyon_wind = grid.get_wind(2, 2);
        assert!(canyon_wind.x > 1.1, "Wind should accelerate in canyon (current: {})", canyon_wind.x);
    }

    #[test]
    fn test_movement_penalty() {
        // Tailwind
        let wind = Vec2::new(10.0, 0.0);
        let move_dir = Vec2::new(1.0, 0.0);
        let cost_mod = calculate_wind_movement_penalty(wind, move_dir);
        assert!(cost_mod < 1.0, "Tailwind should reduce movement cost");

        // Headwind
        let move_dir = Vec2::new(-1.0, 0.0); // Moving West into East wind
        let cost_mod = calculate_wind_movement_penalty(wind, move_dir);
        assert!(cost_mod > 1.0, "Headwind should increase movement cost");

        // Crosswind
        let move_dir = Vec2::new(0.0, 1.0); // Moving North
        let cost_mod = calculate_wind_movement_penalty(wind, move_dir);
        assert!((cost_mod - 1.0).abs() < 0.1, "Pure crosswind should have minimal effect");
    }
}
