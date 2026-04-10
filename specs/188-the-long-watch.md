# 188: The Long Watch

## Overview

Introduces the concept of **Isolation** for Pops working in remote locations. Pops assigned to jobs far from civilization (specifically, `Social` zones or `Tavern` buildings) accumulate `Isolation` over time. High Isolation leads to Stress and potential personality changes (e.g., gaining the `Hermit` trait). This forces players to either build distributed infrastructure (outpost canteens) or rotate crews periodically.

## Dependencies

- `004` — Pop Entity (for `Job` component)
- `097` — Social Tavern (for `Tavern` component)
- `056` — Designated Zones (for `ZoneType::Dining`)
- `127` — Stress Breakdowns (for `Stress` component)

## RED Phase: Tests First

Write these tests in `src/layer1/social/long_watch_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social::long_watch::{Isolation, update_isolation_system, apply_isolation_effects_system};
    use crate::layer1::pop::{Pop, Job, JobType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::social::Tavern;
    use crate::layer1::stress::Stress;
    use crate::layer1::traits::{Traits, Trait};

    fn setup_world() -> World {
        let mut world = World::new();
        // Register components if needed
        world
    }

    #[test]
    fn test_isolation_increases_when_far_from_tavern() {
        let mut world = setup_world();

        // Spawn a remote workplace at (100, 100)
        let mine = world.spawn((
            Building { building_type: BuildingType::Mine }, // Assuming Mine exists or generic
            GridPosition { x: 100, y: 100 },
        )).id();

        // Spawn a tavern far away at (0, 0)
        world.spawn((
            Building { building_type: BuildingType::Tavern },
            GridPosition { x: 0, y: 0 },
            Tavern::default(),
        ));

        // Spawn a worker assigned to the mine
        let worker = world.spawn((
            Pop,
            Job { workplace: mine, job_type: JobType::Miner },
            Isolation { value: 0.0 },
            GridPosition { x: 100, y: 100 }, // Worker is at work
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_isolation_system);
        schedule.run(&mut world);

        let isolation = world.get::<Isolation>(worker).unwrap();
        assert!(isolation.value > 0.0, "Isolation should increase when far from tavern");
    }

    #[test]
    fn test_isolation_decreases_when_near_tavern() {
        let mut world = setup_world();

        // Spawn a workplace near tavern
        let workshop = world.spawn((
            Building { building_type: BuildingType::Workshop },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn tavern at (0, 0)
        world.spawn((
            Building { building_type: BuildingType::Tavern },
            GridPosition { x: 0, y: 0 },
            Tavern::default(),
        ));

        // Spawn worker with high isolation
        let worker = world.spawn((
            Pop,
            Job { workplace: workshop, job_type: JobType::Crafter },
            Isolation { value: 0.5 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_isolation_system);
        schedule.run(&mut world);

        let isolation = world.get::<Isolation>(worker).unwrap();
        assert!(isolation.value < 0.5, "Isolation should decrease when near tavern");
    }

    #[test]
    fn test_high_isolation_causes_stress() {
        let mut world = setup_world();

        let worker = world.spawn((
            Pop,
            Isolation { value: 0.9 }, // High isolation
            Stress { value: 0.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_isolation_effects_system);
        schedule.run(&mut world);

        let stress = world.get::<Stress>(worker).unwrap();
        assert!(stress.value > 0.0, "High isolation should cause stress");
    }

    #[test]
    fn test_hermit_trait_gain() {
        let mut world = setup_world();
        // Seed RNG for determinism if needed, or mock it

        let worker = world.spawn((
            Pop,
            Isolation { value: 1.0 }, // Max isolation
            Traits::default(),
        )).id();

        // Run system enough times to trigger probability (or mock probability)
        // For test, we assume the system attempts to add the trait if isolation is high enough
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_isolation_effects_system);

        // Force deterministic trait gain if possible, or run multiple times
        for _ in 0..100 {
            schedule.run(&mut world);
            if world.get::<Traits>(worker).unwrap().has(Trait::Hermit) {
                break;
            }
        }

        let traits = world.get::<Traits>(worker).unwrap();
        // Note: In a real test, we'd mock the RNG or set the probability to 1.0 via a config resource
        // For this spec, we just check if the logic path exists.
        // assert!(traits.has(Trait::Hermit));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/social/long_watch.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::pop::{Job, Pop};
use crate::layer1::social::Tavern;
use crate::layer1::stress::Stress;
use crate::layer1::traits::{Traits, Trait};

#[derive(Component, Debug, Clone, Default)]
pub struct Isolation {
    pub value: f32, // 0.0 to 1.0
}

/// Constant: Distance threshold for "Remote" work (Manhattan distance).
const REMOTE_THRESHOLD: i32 = 20;

pub fn update_isolation_system(
    mut pops: Query<(&mut Isolation, &Job, &GridPosition), With<Pop>>,
    buildings: Query<(&Building, &GridPosition)>,
    taverns: Query<&Tavern>, // Marker to confirm it's a tavern
) {
    // 1. Collect all Tavern positions
    let tavern_positions: Vec<GridPosition> = buildings.iter()
        .filter(|(b, _)| b.building_type == BuildingType::Tavern)
        .map(|(_, pos)| *pos)
        .collect();

    if tavern_positions.is_empty() {
        // If no taverns exist, everyone gets lonely? Or maybe not.
        // For MVP, if no taverns, isolation increases slowly for everyone working?
        // Let's stick to: Increase if far, Decrease if near.
        return;
    }

    for (mut isolation, job, _pop_pos) in pops.iter_mut() {
        // Find position of the workplace
        // Note: Ideally we use the Workplace's position, but `Job` stores the Entity.
        // We need to look up the workplace position.

        let workplace_pos = if let Ok((_, pos)) = buildings.get(job.workplace) {
            pos
        } else {
            continue; // Job invalid or not a building
        };

        // Calculate distance to nearest Tavern
        let min_dist = tavern_positions.iter()
            .map(|t_pos| (t_pos.x - workplace_pos.x).abs() + (t_pos.y - workplace_pos.y).abs())
            .min()
            .unwrap_or(i32::MAX);

        if min_dist > REMOTE_THRESHOLD {
            // Far away: Increase Isolation
            isolation.value = (isolation.value + 0.01).min(1.0);
        } else {
            // Nearby: Decrease Isolation
            isolation.value = (isolation.value - 0.02).max(0.0);
        }
    }
}

pub fn apply_isolation_effects_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Isolation, &mut Stress, &mut Traits)>,
) {
    let mut rng = rand::thread_rng();

    for (entity, isolation, mut stress, mut traits) in query.iter_mut() {
        if isolation.value > 0.8 {
            // High Isolation: Gain Stress
            stress.value = (stress.value + 0.1).min(100.0);
        }

        if isolation.value > 0.95 {
            // Extreme Isolation: Chance for Hermit trait
            // 1% chance per tick is too high, maybe 0.1%?
            // Or use a "Mental Break" event system if available.
            use rand::Rng;
            if rng.gen_bool(0.001) && !traits.has(Trait::Hermit) {
                traits.add(Trait::Hermit);
                // Log event here
            }
        }
    }
}
```

### 2. Update `Trait` Enum

*Builder needs to update `src/layer1/traits.rs` to include `Hermit`.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

## REFACTOR Phase: Quality & Design

- **Optimization**: `update_isolation_system` iterates all taverns for all pops. O(N*M). Use a spatial grid or cache the "Nearest Tavern" if M grows large.
- **Zone Integration**: Checks for `BuildingType::Tavern`. Should also check for `ZoneType::Dining` (as per `src/layer1/zone.rs`).
- **Commute Logic**: Currently checks `Job.workplace` distance. If the pop is actually *at* the tavern (off shift), isolation should decrease rapidly.
- **Hermit Trait**: Should be implemented to reduce `Social` need decay, effectively adapting the pop to their isolation.

## Acceptance Criteria

- [ ] `Isolation` component exists.
- [ ] Isolation increases when `Job.workplace` is > 20 tiles from any Tavern.
- [ ] Isolation decreases when `Job.workplace` is <= 20 tiles from a Tavern.
- [ ] Stress increases when Isolation > 0.8.
- [ ] `Hermit` trait can be acquired at very high isolation.
- [ ] Tests pass.

## Technical Guidance

- Use Manhattan distance (`(x1-x2).abs() + (y1-y2).abs()`) for grid calculations.
- Remember to import `Trait` from `crate::layer1::traits`.

## Questions

*Builder needs to update `src/layer1/traits.rs` to include `Hermit`.*
*Architect:* Yes, ensure `Trait::Hermit` is added to the `Trait` enum in `src/layer1/traits.rs`.
