# 202: Swarm Intelligence

## Overview

Drones are efficient but dumb. However, when working in groups, they share processing power, increasing their efficiency.
This feature introduces a **Swarm Buff** mechanics.
- **Swarm Proximity**: Drones calculate how many other Drones are within a small radius (e.g., 5 tiles).
- **Efficiency Bonus**: For each nearby Drone, `WorkSpeed` increases by a small percentage (e.g., +5%), up to a cap (e.g., +50%).
- **Visuals**: Drones in a swarm might have a different color or particle effect to indicate the link.

## Dependencies

- `116` — Drone Networks (Drones must exist)
- `016` — Utility AI (Work actions)

## RED Phase: Tests First

Write these tests in `src/layer1/swarm_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::drone::Drone;
    use crate::layer1::map::GridPosition;
    use crate::layer1::stats::WorkSpeed; // Assuming stats module exists or similar
    use crate::layer1::swarm::{Swarm, update_swarm_proximity_system};

    #[test]
    fn test_swarm_initialization() {
        let mut world = World::new();
        let drone = world.spawn((
            Drone,
            Swarm::default(),
            GridPosition { x: 0, y: 0 },
            WorkSpeed(1.0),
        )).id();

        let swarm = world.get::<Swarm>(drone).unwrap();
        assert_eq!(swarm.nearby_count, 0);
        assert_eq!(swarm.efficiency_multiplier, 1.0);
    }

    #[test]
    fn test_swarm_proximity_detection() {
        let mut world = World::new();

        // Drone A
        let drone_a = world.spawn((
            Drone,
            Swarm::default(),
            GridPosition { x: 10, y: 10 },
            WorkSpeed(1.0),
        )).id();

        // Drone B (Adjacent)
        world.spawn((
            Drone,
            Swarm::default(),
            GridPosition { x: 11, y: 10 },
            WorkSpeed(1.0),
        ));

        // Drone C (Far away)
        world.spawn((
            Drone,
            Swarm::default(),
            GridPosition { x: 50, y: 50 },
            WorkSpeed(1.0),
        ));

        // Run system
        // We need a map/spatial query structure.
        // For test simplicity, assume the system iterates all Drones O(N^2) or uses a spatial map resource.
        update_swarm_proximity_system(&mut world);

        let swarm_a = world.get::<Swarm>(drone_a).unwrap();
        assert_eq!(swarm_a.nearby_count, 1); // Only Drone B is close
    }

    #[test]
    fn test_swarm_bonus_application() {
        let mut world = World::new();

        // Drone A with 2 neighbors
        let drone_a = world.spawn((
            Drone,
            Swarm { nearby_count: 2, efficiency_multiplier: 1.0 }, // Pre-set for test
            GridPosition { x: 10, y: 10 },
            WorkSpeed(1.0),
        )).id();

        // System to apply multiplier based on count
        // update_swarm_bonus_system(&mut world);
        // Or integrated in proximity system. Let's assume a separate calculation or same system.

        // Manually trigger the logic if it's inside the system we just wrote
        update_swarm_proximity_system(&mut world);

        let work_speed = world.get::<WorkSpeed>(drone_a).unwrap();
        // Base 1.0 + (2 * 0.05) = 1.1
        assert!(work_speed.0 > 1.09 && work_speed.0 < 1.11);
    }

    #[test]
    fn test_swarm_bonus_cap() {
         let mut world = World::new();

        // Drone A with 20 neighbors (Massive swarm)
        let drone_a = world.spawn((
            Drone,
            Swarm { nearby_count: 20, efficiency_multiplier: 1.0 },
            GridPosition { x: 10, y: 10 },
            WorkSpeed(1.0),
        )).id();

        update_swarm_proximity_system(&mut world);

        let work_speed = world.get::<WorkSpeed>(drone_a).unwrap();
        // Cap is +50% -> 1.5
        assert!(work_speed.0 <= 1.51);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/swarm.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug)]
pub struct Swarm {
    pub nearby_count: u8,
    pub efficiency_multiplier: f32,
}

pub const SWARM_RADIUS: i32 = 5;
pub const SWARM_BONUS_PER_DRONE: f32 = 0.05;
pub const SWARM_BONUS_MAX: f32 = 1.5;
```

### 2. Implement Proximity System

```rust
// src/layer1/swarm.rs

pub fn update_swarm_proximity_system(
    mut query: Query<(Entity, &GridPosition, &mut Swarm, &mut WorkSpeed), With<Drone>>,
    other_drones: Query<(Entity, &GridPosition), With<Drone>>,
) {
    // Naive O(N^2) is fine for low drone counts (<100).
    // For scale, use a SpatialHash or GridMap lookup.

    // 1. Snapshot positions
    let positions: Vec<(Entity, GridPosition)> = other_drones.iter()
        .map(|(e, p)| (e, *p))
        .collect();

    for (entity, pos, mut swarm, mut speed) in query.iter_mut() {
        let mut count = 0;

        for (other_e, other_pos) in &positions {
            if *other_e == entity { continue; }

            // Manhattan distance for simplicity, or Euclidean
            let dist = (pos.x - other_pos.x).abs() + (pos.y - other_pos.y).abs();
            if dist <= SWARM_RADIUS {
                count += 1;
            }
        }

        swarm.nearby_count = count as u8;

        // Apply Bonus
        let bonus = (count as f32 * SWARM_BONUS_PER_DRONE).min(SWARM_BONUS_MAX - 1.0);
        swarm.efficiency_multiplier = 1.0 + bonus;

        // Reset speed to base * multiplier
        // NOTE: This assumes WorkSpeed.0 is the FINAL value.
        // Ideally WorkSpeed has a `base` field, but for MVP we might just set it.
        // Better pattern: WorkSpeed { base: 1.0, current: 1.0 }
        // For this spec, assuming simplistic WorkSpeed(f32):
        speed.0 = 1.0 * swarm.efficiency_multiplier;
    }
}
```

### 3. Integrate

Register `Swarm` component on Drone spawn in `src/layer1/drone.rs`.
Register `update_swarm_proximity_system` in `Layer1SystemSet::Simulation`.

## REFACTOR Phase: Quality & Design

- **Optimization**: Use `Grid` spatial partitioning instead of O(N^2) loop if drone count > 50.
- **Component Design**: `WorkSpeed` should likely be a struct with `base` and `modifiers` list to prevent overwriting other buffs.
- **Visual Feedback**: Draw a thin line between connected drones in UI or change their glyph color to bright cyan when buff is active.

## Acceptance Criteria

- [ ] `Swarm` component exists.
- [ ] Drones detect nearby drones within 5 tiles.
- [ ] `WorkSpeed` increases by 5% per neighbor.
- [ ] Bonus is capped at 50%.
- [ ] Tests pass.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
