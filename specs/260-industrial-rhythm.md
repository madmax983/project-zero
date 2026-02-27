# 260: The Industrial Rhythm

## Overview

"The factory is a symphony."

Machines have **Cycle Times** (ticks to complete a task). If adjacent machines are synchronized (finishing their cycles within a small time window), they create **Rhythm**. High Rhythm boosts worker Morale (the satisfying "thrum" of efficiency) and may slightly improve production speed. Discordant rhythm (random finishes) increases Stress.

This encourages players to build synchronized production lines rather than just spamming machines.

## Dependencies

- `006` — Building Placement (Adjacency)
- `066` — Building Work AI (Work cycles)
- `031` — Pop Morale (Effect target)

## RED Phase: Tests First

Write these tests in `src/layer1/tech/rhythm_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::tech::rhythm::{MachineRhythm, RhythmManager, update_rhythm_system};
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_sync_bonus_adjacent_machines() {
        let mut world = World::new();
        world.insert_resource(RhythmManager::default());

        // Spawn Machine A (finishes at tick 100)
        world.spawn((
            Building { building_type: BuildingType::Refinery },
            GridPosition { x: 5, y: 5 },
            MachineRhythm { cycle_end_tick: 100, last_sync_bonus: 0.0 },
        ));

        // Spawn Machine B (finishes at tick 100) - Perfect Sync
        world.spawn((
            Building { building_type: BuildingType::Refinery },
            GridPosition { x: 6, y: 5 },
            MachineRhythm { cycle_end_tick: 100, last_sync_bonus: 0.0 },
        ));

        // Run system at tick 100
        let mut schedule = Schedule::default();
        schedule.add_systems(update_rhythm_system);

        // Mock time
        world.insert_resource(crate::shared::time::SimulationTime { tick: 100 });

        schedule.run(&mut world);

        // Check bonus
        // Both machines should detect the sync event
        let rhythm = world.query::<&MachineRhythm>().iter(&world).next().unwrap();
        assert!(rhythm.last_sync_bonus > 0.0);
    }

    #[test]
    fn test_discord_penalty() {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime { tick: 100 });

        // Machine A finishes now
        world.spawn((
            GridPosition { x: 0, y: 0 },
            MachineRhythm { cycle_end_tick: 100, last_sync_bonus: 0.0 },
        ));

        // Machine B finished long ago (tick 50) - No sync
        world.spawn((
            GridPosition { x: 1, y: 0 },
            MachineRhythm { cycle_end_tick: 50, last_sync_bonus: 0.0 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_rhythm_system);
        schedule.run(&mut world);

        // No bonus (or penalty if implemented)
        let rhythm = world.query::<&MachineRhythm>().iter(&world).next().unwrap();
        assert_eq!(rhythm.last_sync_bonus, 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/tech/rhythm.rs

use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct MachineRhythm {
    pub cycle_end_tick: u64,
    pub last_sync_bonus: f32, // Accumulated rhythm score
}

#[derive(Resource, Default)]
pub struct RhythmManager {
    // Global rhythm tracking if needed
}
```

### 2. System

```rust
use crate::layer1::map::GridPosition;
use crate::shared::time::SimulationTime;

pub fn update_rhythm_system(
    time: Res<SimulationTime>,
    mut query: Query<(&mut MachineRhythm, &GridPosition)>,
    // Need a way to query neighbors. Self-join is hard in Bevy.
    // Use spatial map or N^2 for small sets.
) {
    // Collect data first to avoid borrow issues
    let machines: Vec<(u64, GridPosition)> = query.iter()
        .map(|(r, p)| (r.cycle_end_tick, *p))
        .collect();

    for (mut rhythm, pos) in query.iter_mut() {
        // Only trigger on completion frame
        if rhythm.cycle_end_tick != time.tick {
            continue;
        }

        let mut sync_count = 0;

        for (other_tick, other_pos) in &machines {
            if *other_pos == *pos { continue; } // Skip self

            // Check adjacency
            if pos.distance_chebyshev(*other_pos) <= 1 {
                // Check sync window (e.g. +/- 2 ticks)
                // For GREEN: exact match or very close
                let diff = (*other_tick as i64 - time.tick as i64).abs();
                if diff <= 2 {
                    sync_count += 1;
                }
            }
        }

        if sync_count > 0 {
            rhythm.last_sync_bonus = 10.0 * sync_count as f32;
            // Emit "Thrum" sound/event here
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Global Audio**: The "Rhythm" should actively drive the background music tempo or layers (Dynamic Music System).
- **Worker Buffs**: Workers standing near high-rhythm machines should get `Effect::InTheZone` (Work Speed +20%).
- **Visualization**: Machines pulse visually on cycle end. Synced machines pulse together.

## Acceptance Criteria

- [ ] Machines track cycle completion time.
- [ ] Adjacent machines finishing simultaneously generate Rhythm bonus.
- [ ] Rhythm bonus is stored on component.
- [ ] Tests pass.

## Technical Guidance

- Integrate with `Spec 066`. When `WorkProgress` reaches 100%, set `cycle_end_tick = current_tick`.
- Use `GridPosition::distance_chebyshev` for adjacency.
