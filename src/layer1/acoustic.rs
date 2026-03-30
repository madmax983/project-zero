#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::suboptimal_flops,
    clippy::cast_possible_wrap,
    clippy::collapsible_if
)]

use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

/// Resource storing the noise levels of the map.
///
/// Values range from 0.0 (silent) to 1.0 (deafening).
#[derive(Resource)]
pub struct NoiseMap {
    /// Width of the noise map.
    pub width: usize,
    /// Height of the noise map.
    pub height: usize,
    /// Flat vector of noise values.
    pub values: Vec<f32>,
}

impl NoiseMap {
    /// Creates a new noise map initialized to ambient levels (0.1).
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Grid size overflow or too large");
        assert!(size <= 10_000_000, "Grid size overflow or too large");
        Self {
            width,
            height,
            values: vec![0.1; size],
        }
    }

    /// Gets the noise value at the given coordinates.
    /// Returns 0.0 if out of bounds.
    #[must_use]
    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 0.0;
        }
        self.values[(y as usize) * self.width + (x as usize)]
    }

    /// Sets the noise value at the given coordinates.
    /// Clamps value between 0.0 and 1.0.
    pub fn set(&mut self, x: i32, y: i32, val: f32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.values[(y as usize) * self.width + (x as usize)] = val.clamp(0.0, 1.0);
    }
}

/// Component designating an entity as a source of noise.
#[derive(Component)]
pub struct NoiseSource {
    /// Maximum radius of the noise in tiles.
    pub radius: f32,
    /// Intensity of the noise at the source (0.0 to 1.0).
    pub intensity: f32,
}

/// System to update the noise map based on sources and terrain.
pub fn update_noise_system(
    mut noise_map: ResMut<NoiseMap>,
    terrain: Res<TerrainGrid>,
    pressure: Option<Res<crate::layer1::pressure::PressureGrid>>,
    sources: Query<(&NoiseSource, &GridPosition)>,
) {
    // Reset to ambient noise
    noise_map.values.fill(0.1);

    for (source, pos) in &sources {
        let mut queue = std::collections::VecDeque::new();
        // Item: (x, y, distance, damping_accumulator)
        queue.push_back((pos.x, pos.y, 0.0_f32, 1.0_f32));

        let mut visited = std::collections::HashSet::new();
        visited.insert((pos.x, pos.y));

        while let Some((px, py, dist, damping)) = queue.pop_front() {
            if dist > source.radius {
                continue;
            }

            // Check vacuum condition
            if let Some(ref press) = pressure {
                if press.get(px, py) < 0.1 {
                    // Sound dies instantly in vacuum
                    continue;
                }
            }

            // Apply noise to current cell
            let falloff = (1.0 - (dist / source.radius)).max(0.0);
            let final_strength = source.intensity * falloff * damping;

            if final_strength > 0.0 {
                let current = noise_map.get(px, py);
                let new_val = (current + final_strength).min(1.0);
                noise_map.set(px, py, new_val);
            }

            // Spread to neighbors
            let neighbors = [
                (px - 1, py, 1.0),
                (px + 1, py, 1.0),
                (px, py - 1, 1.0),
                (px, py + 1, 1.0),
                // Diagonals
                (px - 1, py - 1, 1.414),
                (px + 1, py - 1, 1.414),
                (px - 1, py + 1, 1.414),
                (px + 1, py + 1, 1.414),
            ];

            for (nx, ny, cost) in neighbors {
                if nx < 0 || ny < 0 || nx >= noise_map.width as i32 || ny >= noise_map.height as i32
                {
                    continue;
                }

                if !visited.insert((nx, ny)) {
                    continue; // Already visited
                }

                let mut next_damping = damping;
                if let Some(tile) = terrain.get(nx as usize, ny as usize) {
                    match tile {
                        TerrainType::Rock => next_damping *= 0.2,
                        TerrainType::Tree => next_damping *= 0.8,
                        _ => {}
                    }
                }

                let next_dist = dist + cost;
                if next_dist <= source.radius {
                    queue.push_back((nx, ny, next_dist, next_damping));
                }
            }
        }
    }
}

/// System to apply noise effects to pops (reduce leisure).
pub fn apply_noise_effects_system(
    noise_map: Res<NoiseMap>,
    mut pops: Query<(&GridPosition, &mut Needs)>,
) {
    for (pos, mut needs) in &mut pops {
        let noise = noise_map.get(pos.x, pos.y);
        if noise > 0.5 {
            // High noise stresses pops, reducing leisure.
            // 0.01 per tick is quite harsh if running every tick.
            // But aligned with spec idea.
            needs.leisure = (needs.leisure - 0.01 * noise).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_noise_map_initialization() {
        let map = NoiseMap::new(10, 10);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        // Default ambient noise is low (e.g., 0.1 for wind/nature)
        assert!(map.get(0, 0) <= 0.1);
    }

    #[test]
    fn test_noise_source_propagation() {
        let mut world = World::new();
        world.insert_resource(NoiseMap::new(10, 10));

        let width: usize = 10;
        let height = 10;
        let size = width.checked_mul(height).expect("Grid size overflow");
        assert!(size <= 10_000_000, "Grid size too large");
        let tiles = vec![TerrainType::Grass; size];
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles,
        });

        // Spawn a loud machine at (5, 5)
        world.spawn((
            NoiseSource {
                radius: 4.0,
                intensity: 1.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_noise_system);
        schedule.run(&mut world);

        let map = world.resource::<NoiseMap>();

        // Center is max intensity (1.0 + ambient 0.1 clamped to 1.0)
        assert!((map.get(5, 5) - 1.0).abs() < f32::EPSILON);

        // Intensity drops with distance
        assert!(map.get(5, 5) > map.get(7, 5));

        // Edge of radius (dist 4) should be near 0 (plus ambient)
        // At (9,5), dist is 4. falloff is 0.
        // So just ambient 0.1.
        assert!(map.get(9, 5) < 0.2);
    }

    #[test]
    fn test_noise_damping_by_terrain() {
        let mut world = World::new();
        let width: usize = 10;
        let height = 10;
        let size = width.checked_mul(height).expect("Grid size overflow");
        assert!(size <= 10_000_000, "Grid size too large");
        let mut tiles = vec![TerrainType::Grass; size];

        // Place Rock (blocking) at (6, 5) to block sound going to (7, 5)
        tiles[5 * 10 + 6] = TerrainType::Rock;

        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles,
        });
        world.insert_resource(NoiseMap::new(10, 10));

        world.spawn((
            NoiseSource {
                radius: 5.0,
                intensity: 1.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_noise_system);
        schedule.run(&mut world);

        let map = world.resource::<NoiseMap>();

        // Position (7, 5) is behind Rock at (6, 5)
        // Distance from (5,5) to (7,5) is 2.
        // Falloff = 1.0 - 2/5 = 0.6.
        // Intensity = 1.0 * 0.6 = 0.6.
        // Rock damping = 0.2.
        // Final = 0.6 * 0.2 = 0.12.
        // Plus ambient 0.1 => 0.22.
        // Without damping it would be 0.6 + 0.1 = 0.7.
        // Assert < 0.3 covers 0.22.
        assert!(map.get(7, 5) < 0.3);
    }

    #[test]
    fn test_noise_damping_by_trees() {
        let mut world = World::new();
        let width: usize = 10;
        let height = 10;
        let size = width.checked_mul(height).expect("Grid size overflow");
        assert!(size <= 10_000_000, "Grid size too large");
        let mut tiles = vec![TerrainType::Grass; size];

        // Place Tree at (6, 5)
        tiles[5 * 10 + 6] = TerrainType::Tree;

        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles,
        });
        world.insert_resource(NoiseMap::new(10, 10));

        world.spawn((
            NoiseSource {
                radius: 5.0,
                intensity: 1.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_noise_system);
        schedule.run(&mut world);

        let map = world.resource::<NoiseMap>();

        // Tree damping = 0.8
        // Falloff at dist 2 = 0.6
        // Expected = 1.0 * 0.6 * 0.8 = 0.48.
        // Ambient 0.1 => 0.58.
        // Without damping: 0.7.
        // 0.58 < 0.65.
        let noise = map.get(7, 5);
        assert!(noise < 0.65);
        assert!(noise > 0.5); // Confirm it is damped but not as much as rock
    }

    #[test]
    fn test_noise_affects_rest_recovery() {
        let mut world = World::new();
        let mut map = NoiseMap::new(10, 10);

        // Set high noise at (0, 0)
        map.set(0, 0, 0.9);
        world.insert_resource(map);

        // Spawn sleeping pop in noisy area
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    rest: 0.5,
                    leisure: 0.8,
                    ..Default::default()
                }, // Tired
            ))
            .id();

        // Run the system that modifies needs based on environment
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_noise_effects_system);
        schedule.run(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();

        // Leisure should have decreased
        // 0.8 - 0.01 * 0.9 = 0.791
        assert!(needs.leisure < 0.8);

        // And thus morale should be lower
        // Base morale (0.8 + 0.5 + 0.8 + 1.0(default hygiene)) / 4 = 0.775
        // New morale (0.8 + 0.5 + 0.791 + 1.0) / 4 = 0.77275
        assert!(needs.morale() < 0.775);
        // Also satisfies spec assertion
        assert!(needs.morale() < 1.0);
    }
}
