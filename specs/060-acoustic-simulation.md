# 060: Acoustic Simulation

## Overview

Introduces a **Sound Propagation System** to the simulation.
- **NoiseMap**: A grid tracking noise levels (decibels, normalized 0.0-1.0) across the map.
- **NoiseSource**: Component for entities that emit sound (machines, events, loud pops).
- **Damping**: Walls and terrain reduce noise transmission, creating "acoustic shadows".
- **Gameplay**:
  - Sleeping in noisy areas reduces `Rest` recovery rate and lowers `Morale`.
  - Working in loud environments increases `Stress` (if applicable) or reduces focus.
  - Ideally, industrial zones should be separated from residential zones.

This adds a layer of depth to city planning, encouraging players to separate noisy industry from quiet living quarters.

## Dependencies

- `002` — Terrain Grid (provides `GridPosition` and map dimensions)
- `006` — Building Placement (walls block sound)
- `031` — Pop Morale (provides `Needs` context for penalties)
- `056` — Designated Zones (interaction with Bedrooms)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/acoustic_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{GridPosition, NoiseMap, NoiseSource, update_noise_map_system};
    use crate::layer1::pop::{Pop, Needs};

    #[test]
    fn test_noise_map_initialization() {
        let map = NoiseMap::new(10, 10);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        // Default should be silent (0.0)
        assert_eq!(map.get(5, 5), 0.0);
    }

    #[test]
    fn test_noise_source_propagation() {
        let mut world = World::new();
        world.insert_resource(NoiseMap::new(10, 10));

        // Spawn a loud machine at (5, 5)
        world.spawn((
            NoiseSource { radius: 3.0, intensity: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

        // Run the update system
        update_noise_map_system(&mut world);

        let map = world.resource::<NoiseMap>();

        // Center should be loud
        assert!(map.get(5, 5) >= 0.9);

        // Adjacent tile should be somewhat loud
        assert!(map.get(6, 5) > 0.5);

        // Far tile should be quiet
        assert!(map.get(0, 0) < 0.1);
    }

    #[test]
    fn test_wall_damping() {
        let mut world = World::new();
        world.insert_resource(NoiseMap::new(10, 10));
        // Assume we have a TerrainGrid or BuildingTracker to check for walls
        // For this test, we might need to mock the wall check or insert actual walls if possible
        // Let's assume a simplified test where we manually set damping in the map for now,
        // or rely on the system checking for `Building` with `Wall` tag.

        // This test depends on how walls are implemented (BuildingType::Wall?).
        // If too complex to mock here, verify the damping logic in isolation.
    }

    #[test]
    fn test_noise_affects_rest_recovery() {
        let mut world = World::new();
        let mut map = NoiseMap::new(10, 10);
        map.set(0, 0, 0.8); // Very loud at (0,0)
        world.insert_resource(map);

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { rest: 0.5, ..Default::default() }, // Half tired
            // Assume we have a component tracking state like `Activity::Sleeping`
        )).id();

        // Run the system that applies noise effects
        // crate::layer1::acoustic::apply_noise_effects_system(&mut world);

        // Check if rest recovery was penalized
        // This requires knowing the base rate.
        // Alternatively, check if a "DisturbedSleep" modifier was added.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resources and Components

```rust
// src/layer1/acoustic.rs

use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct NoiseMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<f32>,
}

impl NoiseMap {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            tiles: vec![0.0; (width * height) as usize],
        }
    }

    pub fn get(&self, x: u32, y: u32) -> f32 {
        if x >= self.width || y >= self.height { return 0.0; }
        self.tiles[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, val: f32) {
        if x >= self.width || y >= self.height { return; }
        self.tiles[(y * self.width + x) as usize] = val.clamp(0.0, 1.0);
    }
}

#[derive(Component)]
pub struct NoiseSource {
    pub radius: f32,
    pub intensity: f32, // 0.0 to 1.0
}
```

### 2. Implement `update_noise_map_system`

```rust
pub fn update_noise_map_system(
    mut noise_map: ResMut<NoiseMap>,
    sources: Query<(&NoiseSource, &crate::layer1::map::GridPosition)>,
    // Add query for Walls/Damping if implemented
) {
    // 1. Reset map
    noise_map.tiles.fill(0.0);

    // 2. Spread noise
    for (source, pos) in &sources {
        let radius_i = source.radius.ceil() as i32;
        let center_x = pos.x as i32;
        let center_y = pos.y as i32;

        for dy in -radius_i..=radius_i {
            for dx in -radius_i..=radius_i {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > source.radius { continue; }

                let x = center_x + dx;
                let y = center_y + dy;

                if x < 0 || y < 0 || x >= noise_map.width as i32 || y >= noise_map.height as i32 {
                    continue;
                }

                // Linear falloff
                let falloff = (1.0 - (dist / source.radius)).max(0.0);
                let intensity = source.intensity * falloff;

                // Additive noise (logarithmic addition is better but simple addition works for MVP)
                // Or simply take Max level
                let current = noise_map.get(x as u32, y as u32);
                noise_map.set(x as u32, y as u32, current.max(intensity));
            }
        }
    }
}
```

### 3. Implement Effects

```rust
pub fn apply_noise_effects_system(
    noise_map: Res<NoiseMap>,
    mut pops: Query<(&crate::layer1::map::GridPosition, &mut crate::layer1::pop::Needs)>,
) {
    for (pos, mut needs) in &mut pops {
        let noise = noise_map.get(pos.x as u32, pos.y as u32);

        if noise > 0.5 {
            // Apply stress or reduce rest recovery
            // For MVP: Simple morale penalty if awake?
            // Or reduce `rest` if trying to sleep?
            // Needs system handles recovery, this might just add a "Noise" modifier
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Only update `NoiseMap` when sources move or change intensity (dirty flag).
- **Damping**: Implement raycasting or flood-fill to account for walls blocking sound.
- **Visualization**: Add a debug view mode to see noise hotspots (Red = Loud, Blue = Quiet).
- **Integration**: Link to `BuildingType`. Workshops = Loud, Libraries = Quiet.

## Acceptance Criteria

- [ ] `NoiseMap` resource exists and tracks noise levels.
- [ ] `NoiseSource` component emits noise in a radius.
- [ ] `update_noise_map_system` correctly populates the map.
- [ ] Tests pass.
- [ ] `cargo clippy` passes.
- [ ] Feature enables `006` Buildings to have noise profiles.

## Technical Guidance

- Use `bevy_ecs` resources for the map.
- Keep the map dimensions synced with `TerrainGrid`.
- `update_noise_map_system` should run in the `Simulation` schedule, perhaps in `PreUpdate` or before `Pop` behavior.
