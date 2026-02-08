# 053: Lighting System

## Overview

Introduces a **Lighting System** to the simulation. Tiles now have a light level ranging from 0.0 (Pitch Black) to 1.0 (Bright Day).
- **Ambient Light**: Global light level affecting all tiles (e.g., Sun/Moon).
- **Local Sources**: Entities (Buildings, Items, FX) emit light with a radius and intensity.
- **Gameplay Effects**: Darkness reduces movement speed and work efficiency. Prolonged exposure lowers morale.

This adds depth to base building (necessitating lamps/torches) and creates mood.

## Dependencies

- `002` — Terrain Grid (provides `GridPosition` and map dimensions)
- `031` — Pop Morale (provides `Needs` and morale calculation)
- `051` — Pop Skills (provides base work speed/efficiency context)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/lighting_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{LightMap, LightSource, AmbientLight, update_lighting_system};
    use crate::layer1::GridPosition;
    use crate::layer1::pop::{Pop, movement_speed_system, Speed};

    #[test]
    fn test_light_map_initialization() {
        let map = LightMap::new(10, 10);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        assert_eq!(map.tiles.len(), 100);
        // Default should be 0.0 or handle Ambient separately
    }

    #[test]
    fn test_ambient_light_resource() {
        let ambient = AmbientLight::default();
        assert_eq!(ambient.level, 1.0); // Default to full brightness (Day)
    }

    #[test]
    fn test_light_source_component() {
        let source = LightSource { radius: 5.0, intensity: 1.0, color: (255, 255, 255) };
        assert_eq!(source.radius, 5.0);
    }

    #[test]
    fn test_update_lighting_system_ambient() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        world.insert_resource(AmbientLight { level: 0.5 });

        // No sources
        update_lighting_system(&mut world);

        let map = world.resource::<LightMap>();
        // All tiles should be at least ambient level
        for tile in &map.tiles {
            assert!((tile - 0.5).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_update_lighting_system_source() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        world.insert_resource(AmbientLight { level: 0.0 }); // Pitch black

        // Add a light source at (5, 5)
        world.spawn((
            LightSource { radius: 2.0, intensity: 1.0, color: (255, 255, 255) },
            GridPosition { x: 5, y: 5 },
        ));

        update_lighting_system(&mut world);

        let map = world.resource::<LightMap>();

        // Center should be bright
        assert!(map.get(5, 5) >= 0.9);

        // Edge of radius should be dim
        assert!(map.get(7, 5) > 0.0);

        // Far away should be black
        assert!(map.get(0, 0) < 0.1);
    }

    #[test]
    fn test_darkness_affects_speed() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        world.insert_resource(AmbientLight { level: 0.0 }); // Darkness

        // Pop in darkness
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Speed::default(), // Assuming Speed component exists or is part of Pop
        )).id();

        // Run system that updates speed based on light
        // Note: You might need to mock or setup the system that does this check
        // e.g., `apply_lighting_penalties_system`
        super::apply_lighting_penalties_system(&mut world);

        let speed = world.get::<Speed>(pop).unwrap();
        assert!(speed.current < speed.base);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resources and Components

```rust
// src/layer1/map.rs (or lighting.rs)

use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct AmbientLight {
    pub level: f32, // 0.0 to 1.0
}

#[derive(Resource)]
pub struct LightMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<f32>,
}

impl LightMap {
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
pub struct LightSource {
    pub radius: f32,
    pub intensity: f32,
    pub color: (u8, u8, u8), // For future visual use
}
```

### 2. Implement `update_lighting_system`

```rust
// src/layer1/lighting.rs

pub fn update_lighting_system(
    mut light_map: ResMut<LightMap>,
    ambient: Res<AmbientLight>,
    sources: Query<(&LightSource, &GridPosition)>,
) {
    // 1. Reset map to Ambient
    light_map.tiles.fill(ambient.level);

    // 2. Iterate sources and spread light
    for (source, pos) in &sources {
        let radius = source.radius as i32;
        let center_x = pos.x as i32;
        let center_y = pos.y as i32;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > source.radius { continue; }

                let x = center_x + dx;
                let y = center_y + dy;

                if x < 0 || y < 0 || x >= light_map.width as i32 || y >= light_map.height as i32 {
                    continue;
                }

                // Simple linear falloff: 1.0 at center, 0.0 at radius
                let falloff = (1.0 - (dist / source.radius)).max(0.0);
                let intensity = source.intensity * falloff;

                let current = light_map.get(x as u32, y as u32);
                // Additive blending or Max blending?
                // Max blending is simpler and prevents "super bright spots" overlap artifacts
                light_map.set(x as u32, y as u32, current.max(intensity));
            }
        }
    }
}
```

### 3. Implement Penalties

```rust
// src/layer1/lighting.rs

pub fn apply_lighting_penalties_system(
    light_map: Res<LightMap>,
    mut pops: Query<(&GridPosition, &mut Speed, &mut Needs)>, // Assuming Speed is mutable
) {
    for (pos, mut speed, mut needs) in &mut pops {
        let light = light_map.get(pos.x, pos.y);

        if light < 0.2 {
            // Darkness penalty
            speed.current *= 0.5; // Slow movement

            // Morale penalty (increase stress or reduce mood)
            // Implementation depends on Morale system specifics
            // e.g. needs.morale -= 0.1;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Use a `Dirty` component on LightSources. Only update map if a source moved or changed, or if Ambient changed.
- **Occlusion**: Walls should block light. This requires raycasting or shadow casting, which is expensive. For MVP, light passes through walls (simulating ambient bounce or ceiling lights).
- **Color**: Future support for colored lights (Red Alert vs Blue Calm).
- **Visuals**: In TUI, dim characters (gray) for dark tiles, bright (white/bold) for lit tiles.

## Acceptance Criteria

- [ ] `LightMap` resource exists and tracks light levels.
- [ ] `AmbientLight` resource controls baseline.
- [ ] `LightSource` component emits light in a radius.
- [ ] Pops move slower in darkness (< 0.2 light).
- [ ] Tests pass.
- [ ] `cargo clippy` passes.

## Technical Guidance

- Use `bevy_ecs::prelude::*`.
- Keep `LightMap` separate from `TerrainGrid` to allow independent updates.
- Ensure `update_lighting_system` runs *before* `movement_system` and *before* `rendering`.
