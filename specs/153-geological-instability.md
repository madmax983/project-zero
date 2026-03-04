# 153: Geological Instability

## Overview

The planet is active. Mining activity increases local `SeismicStress`. When stress exceeds a threshold, `GeologicalEvents` occur (Tremors, Earthquakes).
- **Tremors**: Cause Stress to pops in the area.
- **Earthquakes**: Damage buildings and may alter terrain (Rock -> Rubble).

This adds a risk/reward mechanic to intense mining operations.

## Dependencies

- `002` — Terrain Grid (for `TerrainType`)
- `112` — Maintenance Debt (for structural damage mechanics, if applicable, or just Health)
- `016` — Resource Production (Mining increases stress)
- `034` — Pop Health (for damage)

## RED Phase: Tests First

Write these tests in `src/layer1/geology_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::geology::{SeismicGrid, GeologicalEvent, add_seismic_stress, check_seismic_events};
    use crate::layer1::GridPosition;
    use crate::layer1::health::Health;

    #[test]
    fn test_seismic_grid_initialization() {
        let grid = SeismicGrid::new(10, 10);
        assert_eq!(grid.get_stress(5, 5), 0.0);
    }

    #[test]
    fn test_mining_increases_stress() {
        let mut world = World::new();
        let mut grid = SeismicGrid::new(10, 10);
        world.insert_resource(grid);

        // Simulate mining at (5,5)
        add_seismic_stress(&mut world, GridPosition { x: 5, y: 5 }, 10.0);

        let grid = world.resource::<SeismicGrid>();
        assert!(grid.get_stress(5, 5) >= 10.0);
        // Stress should diffuse/spread slightly? Or just local for MVP?
        // Let's assume local for MVP.
    }

    #[test]
    fn test_stress_decay() {
        let mut world = World::new();
        let mut grid = SeismicGrid::new(10, 10);
        grid.add_stress(5, 5, 50.0);
        world.insert_resource(grid);

        // Run decay system (simulated)
        crate::layer1::geology::seismic_decay_system(&mut world);

        let grid = world.resource::<SeismicGrid>();
        assert!(grid.get_stress(5, 5) < 50.0);
    }

    #[test]
    fn test_earthquake_event_trigger() {
        let mut world = World::new();
        let mut grid = SeismicGrid::new(10, 10);
        // Set stress above threshold (e.g., 100.0)
        grid.add_stress(5, 5, 150.0);
        world.insert_resource(grid);
        world.insert_resource(Events::<GeologicalEvent>::default());

        // Run check system
        check_seismic_events(&mut world);

        // Check event
        let events = world.resource::<Events<GeologicalEvent>>();
        let mut reader = events.get_reader();
        let emitted: Vec<_> = reader.read(events).collect();

        assert!(!emitted.is_empty());
        match emitted[0] {
            GeologicalEvent::Earthquake { center, .. } => {
                assert_eq!(center.x, 5);
                assert_eq!(center.y, 5);
            },
            _ => panic!("Expected Earthquake"),
        }
    }

    #[test]
    fn test_earthquake_damage() {
        let mut world = World::new();
        // Setup building/victim
        let victim = world.spawn((
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Trigger earthquake effect
        let event = GeologicalEvent::Earthquake {
            center: GridPosition { x: 5, y: 5 },
            magnitude: 5.0
        };
        crate::layer1::geology::apply_geological_event(&mut world, &event);

        // Check damage
        let health = world.get::<Health>(victim).unwrap();
        assert!(health.current < 100.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Data Structures

```rust
// src/layer1/geology.rs

use bevy_ecs::prelude::*;
use crate::layer1::GridPosition;

#[derive(Resource, Default)]
pub struct SeismicGrid {
    width: usize,
    height: usize,
    stress: Vec<f32>,
}

impl SeismicGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            stress: vec![0.0; width * height],
        }
    }

    pub fn get_stress(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return 0.0;
        }
        self.stress[(y as usize) * self.width + (x as usize)]
    }

    pub fn add_stress(&mut self, x: i32, y: i32, amount: f32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let idx = (y as usize) * self.width + (x as usize);
            self.stress[idx] += amount;
        }
    }

    pub fn decay(&mut self, rate: f32) {
        for s in &mut self.stress {
            *s = (*s - rate).max(0.0);
        }
    }
}

#[derive(Event, Debug)]
pub enum GeologicalEvent {
    Tremor { center: GridPosition },
    Earthquake { center: GridPosition, magnitude: f32 },
}
```

### 2. Logic Functions

```rust
// src/layer1/geology.rs

pub const STRESS_THRESHOLD: f32 = 100.0;
pub const DECAY_RATE: f32 = 0.5;

pub fn add_seismic_stress(world: &mut World, pos: GridPosition, amount: f32) {
    let mut grid = world.resource_mut::<SeismicGrid>();
    grid.add_stress(pos.x, pos.y, amount);
}

pub fn seismic_decay_system(mut grid: ResMut<SeismicGrid>) {
    grid.decay(DECAY_RATE);
}

pub fn check_seismic_events(world: &mut World) {
    let mut events = Vec::new();
    let mut grid = world.resource_mut::<SeismicGrid>();

    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y * grid.width + x;
            if grid.stress[idx] > STRESS_THRESHOLD {
                // Trigger event
                // Reset stress (release energy)
                grid.stress[idx] = 0.0;
                events.push(GeologicalEvent::Earthquake {
                    center: GridPosition { x: x as i32, y: y as i32 },
                    magnitude: 5.0, // Simplified
                });
            }
        }
    }

    for event in events {
        world.send_event(event);
    }
}

pub fn apply_geological_event(world: &mut World, event: &GeologicalEvent) {
    match event {
        GeologicalEvent::Earthquake { center, magnitude } => {
            // Apply damage in radius
            // Naive iteration for MVP
            let mut victims = Vec::new();
            let mut query = world.query::<(Entity, &GridPosition, &mut crate::layer1::health::Health)>();

            for (entity, pos, _) in query.iter(world) {
                if pos.x == center.x && pos.y == center.y { // Only exact tile for now
                     victims.push(entity);
                }
            }

            for entity in victims {
                 if let Ok(mut health) = world.get_mut::<crate::layer1::health::Health>(entity) {
                     health.current -= 10.0 * magnitude;
                 }
            }
        },
        _ => {}
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: `SeismicGrid` is dense. If map is large, this is okay (floats are small).
- **Diffusion**: Stress should diffuse to neighbors to prevent single-tile spikes.
- **Visuals**: Add screen shake or rumble sound (`060` Acoustics).
- **Integration**: Hook into `Mining` action (`016`) to call `add_seismic_stress`.

## Acceptance Criteria

- [ ] `SeismicGrid` resource exists.
- [ ] Mining increases stress at location.
- [ ] Stress decays over time.
- [ ] Stress > Threshold triggers `GeologicalEvent`.
- [ ] `GeologicalEvent` causes damage to entities.
- [ ] Tests pass.

## Technical Guidance

- Use `Events::<GeologicalEvent>` to decouple trigger from effect.
- Ensure `SeismicGrid` is initialized with map dimensions in `setup`.
- Be careful with `world.query` inside event handler; iterating all entities is slow. Use spatial partitioning if available.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
