# 009: Job Assignment System

## Overview

Pops need to be assigned to buildings to work or rest. This system automatically assigns idle pops to available jobs based on their needs. It's the brain that makes the colony self-managing.

## Dependencies

- `004` — Pop entity
- `005` — Pop needs
- `007` — Housing building
- `008` — Farm building

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/job.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_pop_state_default() {
        let state = PopState::default();
        assert_eq!(state, PopState::Idle);
    }

    #[test]
    fn test_pop_state_variants() {
        let mut world = World::new();
        let building = world.spawn(()).id();

        let state_idle = PopState::Idle;
        let state_working = PopState::Working(building);
        let state_resting = PopState::Resting(building);

        assert_eq!(state_idle, PopState::Idle);
        assert!(matches!(state_working, PopState::Working(_)));
        assert!(matches!(state_resting, PopState::Resting(_)));
    }

    #[test]
    fn test_assign_jobs_system_assigns_to_farm() {
        let mut world = World::new();

        // Idle pop
        world.spawn((
            Pop,
            Needs { hunger: 0.5, rest: 0.8 },
            PopState::Idle,
        ));

        // Available farm
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
            Farm::default(),
        ));

        assign_jobs_system(&mut world);

        // Pop should be assigned
        let state = world.query::<&PopState>().single(&world);
        assert!(matches!(*state, PopState::Working(_)));
    }

    #[test]
    fn test_assign_jobs_system_assigns_to_housing_when_tired() {
        let mut world = World::new();

        // Tired pop
        world.spawn((
            Pop,
            Needs { hunger: 0.8, rest: 0.2 },
            PopState::Idle,
        ));

        // Available housing
        world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 5, y: 5 },
            Housing::default(),
        ));

        assign_jobs_system(&mut world);

        // Pop should be resting
        let state = world.query::<&PopState>().single(&world);
        assert!(matches!(*state, PopState::Resting(_)));
    }

    #[test]
    fn test_assign_jobs_system_prioritizes_urgent_needs() {
        let mut world = World::new();

        let pop1 = world.spawn((
            Pop,
            Needs { hunger: 0.1, rest: 0.8 }, // Very hungry
            PopState::Idle,
        )).id();

        let pop2 = world.spawn((
            Pop,
            Needs { hunger: 0.6, rest: 0.8 }, // Less hungry
            PopState::Idle,
        )).id();

        // Only one farm slot
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
            Farm { capacity: 1, workers: Vec::new() },
        ));

        assign_jobs_system(&mut world);

        // Most urgent pop should get the slot
        let state1 = world.get::<PopState>(pop1).unwrap();
        let state2 = world.get::<PopState>(pop2).unwrap();

        assert!(matches!(*state1, PopState::Working(_)), "Starving pop should work");
        assert_eq!(*state2, PopState::Idle, "Less urgent pop should stay idle");
    }

    #[test]
    fn test_assign_jobs_system_respects_capacity() {
        let mut world = World::new();

        world.spawn((Pop, Needs::default(), PopState::Idle));
        world.spawn((Pop, Needs::default(), PopState::Idle));
        world.spawn((Pop, Needs::default(), PopState::Idle));

        // Farm with capacity 2
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
            Farm::default(), // capacity 2
        ));

        assign_jobs_system(&mut world);

        let working_count = world.query::<&PopState>()
            .iter(&world)
            .filter(|s| matches!(**s, PopState::Working(_)))
            .count();

        assert_eq!(working_count, 2, "Should not exceed farm capacity");
    }

    #[test]
    fn test_release_satisfied_pops_system() {
        let mut world = World::new();

        let housing = world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 5, y: 5 },
            Housing::default(),
        )).id();

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.8, rest: 0.95 }, // Well rested
            PopState::Resting(housing),
        )).id();

        // Add pop to housing residents
        world.get_mut::<Housing>(housing).unwrap().residents.push(pop);

        release_satisfied_pops_system(&mut world);

        // Pop should be released
        let state = world.get::<PopState>(pop).unwrap();
        assert_eq!(*state, PopState::Idle);

        // Should be removed from housing
        let housing_comp = world.get::<Housing>(housing).unwrap();
        assert!(housing_comp.residents.is_empty());
    }

    #[test]
    fn test_release_critical_workers() {
        let mut world = World::new();

        let farm = world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
            Farm::default(),
        )).id();

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.1, rest: 0.1 }, // Critical needs
            PopState::Working(farm),
        )).id();

        world.get_mut::<Farm>(farm).unwrap().workers.push(pop);

        release_satisfied_pops_system(&mut world);

        // Pop should be released due to critical needs
        let state = world.get::<PopState>(pop).unwrap();
        assert_eq!(*state, PopState::Idle);
    }

    #[test]
    fn test_pop_display_with_state_idle() {
        let needs = Needs { hunger: 0.8, rest: 0.8 };
        let state = PopState::Idle;

        let (ch, color) = pop_display(&needs, &state);
        assert_eq!(ch, '☺');
    }

    #[test]
    fn test_pop_display_with_state_working() {
        let mut world = World::new();
        let farm = world.spawn(()).id();

        let needs = Needs { hunger: 0.8, rest: 0.8 };
        let state = PopState::Working(farm);

        let (ch, _) = pop_display(&needs, &state);
        assert_eq!(ch, '⚒');
    }

    #[test]
    fn test_pop_display_with_state_resting() {
        let mut world = World::new();
        let housing = world.spawn(()).id();

        let needs = Needs { hunger: 0.8, rest: 0.8 };
        let state = PopState::Resting(housing);

        let (ch, _) = pop_display(&needs, &state);
        assert_eq!(ch, '☻');
    }

    #[test]
    fn test_pop_display_extended_uses_health_for_color() {
        let mut world = World::new();
        let farm = world.spawn(()).id();

        let needs_healthy = Needs { hunger: 0.8, rest: 0.8 };
        let needs_critical = Needs { hunger: 0.2, rest: 0.8 };
        let state = PopState::Working(farm);

        let (_, color_healthy) = pop_display(&needs_healthy, &state);
        let (_, color_critical) = pop_display(&needs_critical, &state);

        assert_eq!(color_healthy, Color::Yellow);
        assert_eq!(color_critical, Color::Red);
    }

    #[test]
    fn test_full_simulation_cycle() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn pops
        for _ in 0..3 {
            world.spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs::default(),
                PopState::Idle,
            ));
        }

        // Spawn buildings
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 10, y: 10 },
            Farm::default(),
        ));

        world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 15, y: 15 },
            Housing::default(),
        ));

        // Run simulation
        for _ in 0..10 {
            release_satisfied_pops_system(&mut world);
            assign_jobs_system(&mut world);
            produce_food_system(&mut world);
            restore_rest_in_housing_system(&mut world);
            consume_food_system(&mut world);
            decay_needs_system(&mut world);
        }

        // All pops should still be alive
        let pop_count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(pop_count, 3, "Colony should be stable");

        // Should have some food
        let resources = world.resource::<ColonyResources>();
        assert!(resources.food > 0.0, "Should have accumulated food");
    }
}
```

**Test Coverage Requirements:**
- PopState: default, all variants
- assign_jobs_system: farm assignment, housing assignment, priority, capacity
- release_satisfied_pops_system: release from housing, release from work
- pop_display extended: character based on state, color based on health
- Integration: full simulation cycle remains stable
- Coverage ≥85% for layer1/job.rs
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### PopState Component

```rust
// src/layer1/job.rs

use bevy_ecs::prelude::*;

/// Pop job state - tracks current activity.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PopState {
    #[default]
    Idle,
    Working(Entity),  // Entity = the farm
    Resting(Entity),  // Entity = the housing
}
```

### Assignment System

```rust
// src/layer1/job.rs

use super::needs::Needs;
use super::pop::Pop;
use crate::layer2::housing::Housing;
use crate::layer2::farm::Farm;

/// Assigns idle pops to available jobs based on their needs.
pub fn assign_jobs_system(world: &mut World) {
    // Collect idle pops with their needs
    let mut idle_pops: Vec<(Entity, f32, f32)> = world
        .query_filtered::<(Entity, &Needs, &PopState), With<Pop>>()
        .iter(world)
        .filter(|(_, _, state)| **state == PopState::Idle)
        .map(|(e, n, _)| (e, n.hunger, n.rest))
        .collect();

    // Sort by urgency (lowest need value = most urgent)
    idle_pops.sort_by(|a, b| {
        let a_worst = a.1.min(a.2);
        let b_worst = b.1.min(b.2);
        a_worst.partial_cmp(&b_worst).unwrap()
    });

    for (pop_entity, hunger, rest) in idle_pops {
        let needs_rest = rest < 0.3;
        let needs_food = hunger < 0.3;

        if needs_rest {
            // Try to assign to housing
            if try_assign_to_housing(world, pop_entity) {
                continue;
            }
        }

        // Default or needs_food: try to assign to farm
        if try_assign_to_farm(world, pop_entity) {
            continue;
        }

        // If still idle and needs rest badly, try housing even if not critical
        if rest < 0.5 {
            try_assign_to_housing(world, pop_entity);
        }
    }
}

fn try_assign_to_housing(world: &mut World, pop: Entity) -> bool {
    // Find housing with space
    let available: Option<Entity> = world
        .query::<(Entity, &Housing)>()
        .iter(world)
        .find(|(_, h)| h.residents.len() < h.capacity)
        .map(|(e, _)| e);

    if let Some(housing_entity) = available {
        // Add to housing
        if let Some(mut housing) = world.get_mut::<Housing>(housing_entity) {
            housing.residents.push(pop);
        }
        // Update pop state
        if let Some(mut state) = world.get_mut::<PopState>(pop) {
            *state = PopState::Resting(housing_entity);
        }
        return true;
    }
    false
}

fn try_assign_to_farm(world: &mut World, pop: Entity) -> bool {
    // Find farm with space
    let available: Option<Entity> = world
        .query::<(Entity, &Farm)>()
        .iter(world)
        .find(|(_, f)| f.workers.len() < f.capacity)
        .map(|(e, _)| e);

    if let Some(farm_entity) = available {
        // Add to farm
        if let Some(mut farm) = world.get_mut::<Farm>(farm_entity) {
            farm.workers.push(pop);
        }
        // Update pop state
        if let Some(mut state) = world.get_mut::<PopState>(pop) {
            *state = PopState::Working(farm_entity);
        }
        return true;
    }
    false
}
```

### Release System

```rust
// src/layer1/job.rs

/// Releases pops from jobs when needs are satisfied or critical.
pub fn release_satisfied_pops_system(world: &mut World) {
    // Collect pops to potentially release
    let check_pops: Vec<(Entity, PopState, f32, f32)> = world
        .query::<(Entity, &PopState, &Needs)>()
        .iter(world)
        .filter(|(_, state, _)| **state != PopState::Idle)
        .map(|(e, s, n)| (e, *s, n.hunger, n.rest))
        .collect();

    for (pop_entity, state, hunger, rest) in check_pops {
        let should_release = match state {
            PopState::Resting(housing_entity) => {
                if rest > 0.9 {
                    // Fully rested, release
                    if let Some(mut housing) = world.get_mut::<Housing>(housing_entity) {
                        housing.residents.retain(|&e| e != pop_entity);
                    }
                    true
                } else {
                    false
                }
            }
            PopState::Working(farm_entity) => {
                // Release if needs are critical
                if hunger < 0.2 || rest < 0.2 {
                    if let Some(mut farm) = world.get_mut::<Farm>(farm_entity) {
                        farm.workers.retain(|&e| e != pop_entity);
                    }
                    true
                } else {
                    false
                }
            }
            PopState::Idle => false,
        };

        if should_release {
            if let Some(mut state) = world.get_mut::<PopState>(pop_entity) {
                *state = PopState::Idle;
            }
        }
    }
}
```

### Extended Pop Display

```rust
// src/layer1/pop.rs - Replace pop_display function

use super::needs::Needs;
use super::job::PopState;
use ratatui::style::Color;

/// Returns the character and color for rendering a pop based on their needs and state.
///
/// **This is the extended version from spec 009.** Spec 005 used `pop_display(&Needs)`.
/// This version adds PopState to show different characters for working/resting pops.
#[must_use]
pub fn pop_display(needs: &Needs, state: &PopState) -> (char, Color) {
    let health = needs.worst();

    let base_color = if health > 0.6 {
        Color::Yellow
    } else if health > 0.3 {
        Color::Rgb(255, 165, 0) // Orange
    } else {
        Color::Red
    };

    let ch = match state {
        PopState::Idle => '☺',
        PopState::Working(_) => '⚒',
        PopState::Resting(_) => '☻',
    };

    (ch, base_color)
}
```

### Update Pop Spawning

```rust
// src/layer1/pop.rs - Modify spawn_initial_pops

use super::job::PopState;

pub fn spawn_initial_pops(world: &mut World) {
    let terrain = world.resource::<TerrainGrid>();
    let mut rng = rand::thread_rng();
    let mut spawned = 0;

    while spawned < 5 {
        let x = rng.gen_range(0..terrain.width as i32);
        let y = rng.gen_range(0..terrain.height as i32);

        if let Some(terrain_type) = terrain.get(x as usize, y as usize) {
            if terrain_type != TerrainType::Water && terrain_type != TerrainType::Rock {
                world.spawn((
                    Pop,
                    GridPosition { x, y },
                    Needs::default(),
                    PopState::default(), // ADD THIS
                ));
                spawned += 1;
            }
        }
    }
}
```

### Update Rendering

```rust
// src/main.rs - Update render_map

fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    // ... existing code ...

    // Collect pops WITH STATE
    let pops_data: Vec<(GridPosition, (char, Color))> = world
        .query::<(&GridPosition, &Needs, &PopState)>()
        .iter(world)
        .map(|(pos, needs, state)| (*pos, pop_display(needs, state)))
        .collect();

    // ... rest of rendering ...
}
```

### System Execution Order

```rust
// src/main.rs - Update simulation tick

if *world.resource::<GameState>() == GameState::Running {
    let speed = world.resource::<SimulationTime>().speed;
    if speed != SimSpeed::Paused {
        // Job management (FIRST)
        release_satisfied_pops_system(&mut world);
        assign_jobs_system(&mut world);

        // Production and restoration
        produce_food_system(&mut world);
        restore_rest_in_housing_system(&mut world);

        // Consumption
        consume_food_system(&mut world);

        // Decay
        decay_needs_system(&mut world);

        // Death and cleanup
        kill_starving_pops_system(&mut world);
        clean_dead_workers_system(&mut world);
        clean_dead_residents_system(&mut world);

        world.resource_mut::<SimulationTime>().tick += 1;
    }
}
```

### Module Integration

```rust
// src/layer1/mod.rs
pub mod terrain;
pub mod pop;
pub mod needs;
pub mod job;

pub use terrain::{TerrainGrid, TerrainType, Viewport, generate_terrain};
pub use pop::{Pop, GridPosition, spawn_initial_pops, pop_display};
pub use needs::{Needs, decay_needs_system, kill_starving_pops_system};
pub use job::{PopState, assign_jobs_system, release_satisfied_pops_system};
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Priority is simple min()**: Doesn't weight hunger vs rest differently
   - Future: Could prioritize hunger more (starvation is fatal)
   - Current: Simple urgency-based assignment works

2. **No job preferences**: Pops don't have likes/dislikes
   - Future: Add personality traits or skills
   - Current: All pops are identical workers

3. **Release thresholds hardcoded**: 0.9 for rest, 0.2 for critical
   - Future: Extract to constants or make configurable
   - Current: Values create good gameplay feel

4. **No commute time**: Pops teleport to jobs
   - Future: Add pathfinding and travel time
   - Current: Instant assignment acceptable for colony sim

5. **Binary assignment**: Pop is either working or not
   - Future: Add partial work, efficiency based on morale
   - Current: Simple binary state sufficient

### System Execution Order (Critical!)

**Correct order:**
1. `release_satisfied_pops_system` - Free up job slots FIRST
2. `assign_jobs_system` - Fill available slots
3. `produce_food_system` - Workers generate food
4. `restore_rest_in_housing_system` - Resting pops recover
5. `consume_food_system` - Everyone eats
6. `decay_needs_system` - Needs decrease
7. `kill_starving_pops_system` - Despawn dead
8. `clean_dead_workers_system` - Remove references
9. `clean_dead_residents_system` - Remove references

**Why this order:**
- Release before assign: Prevents blocking when pops should switch jobs
- Assign before production: Ensures workers are in place to produce
- Production/restoration before consumption/decay: Resources available before needs drop
- Death before cleanup: Ensures all dead entities cleaned up

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for layer1/job.rs
- [x] Idle pops automatically get assigned to buildings
- [x] Starving pops (hunger < 0.3) prioritize farms
- [x] Exhausted pops (rest < 0.3) prioritize housing
- [x] Pops release from housing when rested (> 0.9)
- [x] Pops release from work when needs critical (< 0.2)
- [x] Pop character changes based on state (☺ idle / ⚒ working / ☻ resting)
- [x] With 1 farm + 1 housing + 5 pops, colony stabilizes
- [x] PopState component reflects current activity

## Technical Guidance

### Assignment Priority Logic

```
For each idle pop (sorted by worst need):
  1. If rest < 0.3: Try housing
  2. Otherwise: Try farm
  3. If still idle and rest < 0.5: Try housing again
```

This creates:
- **Emergency triage**: Critical rest gets immediate housing
- **Default work**: Pops work farms unless tired
- **Fallback rest**: Moderately tired pops seek housing if farm full

### Release Conditions

| State | Release When | Reason |
|-------|--------------|--------|
| Resting | rest > 0.9 | Fully rested |
| Working | hunger < 0.2 OR rest < 0.2 | Critical need |
| Idle | Never | Already idle |

### Victory Condition

With specs 001-009 complete:
1. Start game → see terrain and 5 pops
2. Pops begin dying (no food!)
3. Press B, place farm → pops auto-assign, food appears
4. Place housing → tired pops rest
5. Colony stabilizes if buildings sufficient
6. Colony dies if buildings insufficient

**You have a playable game.**

### Common Pitfalls

1. **Wrong system order**: Assign after production = no workers that tick
2. **Forgetting PopState on spawn**: Pops never leave Idle (panic!)
3. **Not updating pop_display call sites**: Still using old signature
4. **Expecting instant equilibrium**: Takes ~10-20 ticks to stabilize

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

## Forward Compatibility Note

**pop_display Signature Evolution Complete:**

- **Spec 005**: `pop_display(needs: &Needs) -> (char, Color)` (basic version)
- **Spec 009**: `pop_display(needs: &Needs, state: &PopState) -> (char, Color)` (extended version)

All call sites must be updated to pass PopState. Tests in spec 005 used the basic version; tests in this spec validate the extended version.
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
