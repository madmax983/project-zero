# 060: Acoustic Simulation

## Overview

Introduces sound propagation and noise levels to the colony simulation. The colony is no longer silent; machines hum, mining blasts echo, and pops converse.

- **NoiseMap**: A dynamic grid tracking noise levels (decibels normalized 0.0 to 1.0).
- **NoiseSource**: Component for entities that emit sound (e.g., Machines, Pops).
- **Damping**: Terrain (Rock, Trees) and walls reduce noise spread, creating "Acoustic Zones".
- **Gameplay**:
    - High noise levels in `Bedroom` zones reduce `Rest` recovery rate and lower `Morale`.
    - `Library` and `Chapel` buildings (future) will require low noise levels.
    - Separating industrial areas from living quarters becomes a mechanical necessity.

## Dependencies

- `002` Terrain Grid (for map dimensions and damping).
- `006` Building Placement (walls block sound - future).
- `056` Designated Zones (interaction with Bedroom/Hospital).
- `005` Pop Needs (Rest recovery).
- `031` Pop Morale (Mood penalties).

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/acoustic_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{GridPosition, TerrainGrid, TerrainType};
    use crate::layer1::acoustic::{NoiseMap, NoiseSource, update_noise_system, apply_noise_effects_system};
    use crate::layer1::pop::{Pop, Needs};

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
        // Empty terrain (Grass) does not dampen much
        world.insert_resource(TerrainGrid::new(10, 10));

        // Spawn a loud machine at (5, 5)
        world.spawn((
            NoiseSource { radius: 4.0, intensity: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

        update_noise_system(&mut world);

        let map = world.resource::<NoiseMap>();

        // Center is max intensity
        assert!((map.get(5, 5) - 1.0).abs() < f32::EPSILON);

        // Intensity drops with distance
        assert!(map.get(5, 5) > map.get(7, 5));

        // Edge of radius is near 0 (plus ambient)
        assert!(map.get(9, 5) < 0.2);
    }

    #[test]
    fn test_noise_damping_by_terrain() {
        let mut world = World::new();
        let mut terrain = TerrainGrid::new(10, 10);

        // Place Rock (blocking) at (6, 5) to block sound going to (7, 5)
        // Note: TerrainGrid::new fills with Grass. We need to manually set Rock.
        // Assuming TerrainGrid has a set method or public tiles.
        terrain.tiles[5 * 10 + 6] = TerrainType::Rock;

        world.insert_resource(terrain);
        world.insert_resource(NoiseMap::new(10, 10));

        world.spawn((
            NoiseSource { radius: 5.0, intensity: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

        update_noise_system(&mut world);

        let map = world.resource::<NoiseMap>();

        // Position (7, 5) is behind Rock at (6, 5)
        // It should be significantly quieter than if Rock wasn't there.
        // Without damping: dist=2, intensity ~ 0.6
        // With damping: expected < 0.3
        assert!(map.get(7, 5) < 0.3);
    }

    #[test]
    fn test_noise_affects_rest_recovery() {
        let mut world = World::new();
        let mut map = NoiseMap::new(10, 10);

        // Set high noise at (0, 0)
        map.set(0, 0, 0.9);
        world.insert_resource(map);

        // Spawn sleeping pop in noisy area
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { rest: 0.5, ..Default::default() }, // Tired
        )).id();

        // Run the system that modifies needs based on environment
        apply_noise_effects_system(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        // Assume morale is impacted
        assert!(needs.morale < 1.0); // Assuming it started max
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components and Resources

```rust
// src/layer1/acoustic.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};

#[derive(Resource)]
pub struct NoiseMap {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl NoiseMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.1; width * height], // 0.1 ambient
        }
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 0.0;
        }
        self.values[(y as usize) * self.width + (x as usize)]
    }

    pub fn set(&mut self, x: i32, y: i32, val: f32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.values[(y as usize) * self.width + (x as usize)] = val.clamp(0.0, 1.0);
    }
}

#[derive(Component)]
pub struct NoiseSource {
    pub radius: f32,
    pub intensity: f32, // 0.0 - 1.0
}
```

### 2. Implement Propagation System

```rust
pub fn update_noise_system(
    mut noise_map: ResMut<NoiseMap>,
    terrain: Res<TerrainGrid>,
    sources: Query<(&NoiseSource, &GridPosition)>,
) {
    // Reset to ambient
    noise_map.values.fill(0.1);

    for (source, pos) in &sources {
        let r = source.radius.ceil() as i32;
        let cx = pos.x;
        let cy = pos.y;

        for dy in -r..=r {
            for dx in -r..=r {
                let dist_sq = (dx*dx + dy*dy) as f32;
                if dist_sq > source.radius * source.radius { continue; }

                // Check for blocking terrain between source and target
                // For MVP, we just check the target tile itself for damping
                // True occlusion requires raycasting which is complex for Green phase.
                // We will implement a simplified model:
                // If target tile is Rock, sound is heavily damped.
                // If intermediate tiles are Rock, sound is damped (omitted for MVP simplicity unless test fails).

                // Wait, the test expects (7,5) to be quiet because (6,5) is Rock.
                // This REQUIRES line-of-sight or path checking.
                // Simple implementation: "Bresenham's line" check or similar.

                let tx = cx + dx;
                let ty = cy + dy;

                if tx < 0 || ty < 0 || tx >= noise_map.width as i32 || ty >= noise_map.height as i32 {
                    continue;
                }

                // Check Line of Sight for damping
                let mut damping = 1.0;

                // Simple raycast from source to target
                let steps = dist_sq.sqrt().ceil() as i32;
                if steps > 0 {
                    let step_x = dx as f32 / steps as f32;
                    let step_y = dy as f32 / steps as f32;

                    for s in 1..=steps {
                        let ix = (cx as f32 + step_x * s as f32).round() as i32;
                        let iy = (cy as f32 + step_y * s as f32).round() as i32;

                        if let Some(tile) = terrain.get(ix as usize, iy as usize) {
                            if tile == TerrainType::Rock {
                                damping *= 0.2; // Rock blocks 80% of sound
                            } else if tile == TerrainType::Tree {
                                damping *= 0.8; // Trees block 20%
                            }
                        }
                    }
                }

                let dist = dist_sq.sqrt();
                // Linear falloff with damping
                let raw_strength = source.intensity * (1.0 - (dist / source.radius));
                let final_strength = raw_strength * damping;

                if final_strength > 0.0 {
                    let current = noise_map.get(tx, ty);
                    let new_val = (current + final_strength * 0.5).min(1.0);
                    noise_map.set(tx, ty, new_val);
                }
            }
        }
    }
}
```

### 3. Implement Effects System

```rust
pub fn apply_noise_effects_system(
    noise_map: Res<NoiseMap>,
    mut pops: Query<(&GridPosition, &mut crate::layer1::pop::Needs)>,
) {
    for (pos, mut needs) in &mut pops {
        let noise = noise_map.get(pos.x, pos.y);
        if noise > 0.5 {
            // High noise stresses pops
            needs.morale -= 0.01 * noise;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: `NoiseMap` recalculation with raycasting is O(N_sources * R^3).
    - *Improvement*: Pre-calculate a "DampingMap" or "VisibilityMap" if terrain is static.
    - *Improvement*: Use BFS flood fill instead of raycasting for correct diffraction around corners (sound doesn't just go in straight lines).
- **Integration**:
    - Add `NoiseSource` to `Machine` buildings.
    - Add `NoiseSource` to `Mining` action (temporary entity).
- **Visualization**: Add a debug view (overlay) for NoiseMap in the TUI/GUI.

## Acceptance Criteria

- [ ] `NoiseMap` resource exists and tracks values.
- [ ] `NoiseSource` component is defined.
- [ ] Noise propagates from source position with distance falloff.
- [ ] Terrain (Rock) dampens noise propagation significantly.
- [ ] Pops in high-noise areas suffer Morale penalties.
- [ ] All tests pass.
- [ ] `cargo clippy` passes.

## Technical Guidance

- Use `bevy_ecs` resources for the grid.
- Ensure `update_noise_system` runs before `pop_update_system`.
- Be careful with casting `usize` to `i32` for grid coordinates.
