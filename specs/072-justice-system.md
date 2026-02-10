# 072: Justice System

## Overview

Introduces Law & Order mechanics to the colony. Pops can be marked as `Wanted` for committing crimes (specifically Vandalism from Spec 050). A new `ActionType::Warden` allows designated guards (Pops with high Warden weight) to arrest these targets and escort them to a `Jail` Zone (Spec 056). Inmates serve a sentence and are then released, removing the Wanted status.

## Dependencies

- `050` — Civil Unrest (Crime source: Vandalism)
- `056` — Designated Zones (Jail Zone)
- `067` — Militia System (Combat mechanics for arrest)
- `016` — Utility AI (Warden action evaluation)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/justice_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Wanted, Inmate, MentalState, MentalBreakType};
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::justice::{check_crime_system, warden_evaluation_system, execute_arrest_system};
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::map::GridPosition;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(ZoneGrid::new(10, 10));
        world
    }

    // 1. Crime Detection
    #[test]
    fn test_vandalism_triggers_wanted_status() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop,
            MentalState::Broken(MentalBreakType::Vandalize),
            // Not yet Wanted
        )).id();

        // Run detection
        check_crime_system(&mut world);

        // Should be marked Wanted
        assert!(world.get::<Wanted>(pop).is_some());
    }

    // 2. Warden Action Evaluation
    #[test]
    fn test_warden_evaluates_arrest() {
        let mut world = setup_world();

        // Criminal
        let criminal = world.spawn((
            Pop,
            Wanted { severity: 1.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Guard
        let guard = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
            crate::layer1::utility_ai::UtilityWeights::default(), // Warden weight implied default
        )).id();

        // Run evaluation logic (simulated)
        let (score, target) = crate::layer1::justice::evaluate_warden_action(&world, &GridPosition { x: 0, y: 0 }).unwrap();

        assert!(score > 0.0);
        assert_eq!(target, criminal);
    }

    // 3. Arrest Execution
    #[test]
    fn test_arrest_execution_converts_to_inmate() {
        let mut world = setup_world();

        // Define Jail Zone
        world.resource_mut::<ZoneGrid>().set(2, 2, ZoneType::Jail);

        let criminal = world.spawn((
            Pop,
            Wanted { severity: 1.0 },
            GridPosition { x: 1, y: 1 }, // Next to guard
        )).id();

        let guard = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 }, // Adjacent
        )).id();

        // Simulate successful arrest action
        execute_arrest_system(&mut world, guard, criminal);

        // Criminal should be Inmate, moved to Jail (conceptually, or just state change)
        assert!(world.get::<Inmate>(criminal).is_some());
        assert!(world.get::<Wanted>(criminal).is_none());

        // Position check (teleport for MVP, escort for Refactor)
        let pos = world.get::<GridPosition>(criminal).unwrap();
        assert_eq!(pos.x, 2);
        assert_eq!(pos.y, 2);
    }

    #[test]
    fn test_inmate_sentence_decay() {
        let mut world = setup_world();
        let inmate = world.spawn((
            Pop,
            Inmate { sentence_ticks: 1 },
        )).id();

        // Update time/inmates
        crate::layer1::justice::update_inmates_system(&mut world);

        // Should be free
        assert!(world.get::<Inmate>(inmate).is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

`src/layer1/justice.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Wanted {
    pub severity: f32, // 0.0 to 1.0
}

#[derive(Component, Debug, Default)]
pub struct Inmate {
    pub sentence_ticks: u32,
}
```

### 2. Update `ActionType`

`src/layer1/utility_ai/types.rs`:

```rust
pub enum ActionType {
    // ...
    Warden, // 16 (Total 17)
}

impl ActionType {
    pub const COUNT: usize = 17;

    // ...
    Self::Warden => 16,
}
```

**CRITICAL**: Update `src/gpu/buffers.rs` `GpuPopInput`:
- `success_count: [u32; 17]`
- `attempt_count: [u32; 17]`
- `_padding: [u32; 1]` (to reach 176 bytes)
- Ensure total size is still 176 bytes.

### 3. Update `ZoneType`

`src/layer1/zone.rs`:

```rust
pub enum ZoneType {
    // ...
    Jail,
}
```

### 4. Logic Implementation

`src/layer1/justice.rs`:

```rust
pub fn check_crime_system(
    mut commands: Commands,
    query: Query<(Entity, &MentalState), Without<Wanted>>,
) {
    for (entity, state) in query.iter() {
        if let MentalState::Broken(MentalBreakType::Vandalize) = state {
            commands.entity(entity).insert(Wanted { severity: 1.0 });
        }
    }
}

pub fn evaluate_warden_action(
    world: &World,
    guard_pos: &GridPosition,
) -> Option<(f32, Entity)> {
    // Find nearest Wanted pop
    let mut best_target = None;
    let mut min_dist = f32::MAX;

    // Naive query for MVP
    let mut query = world.query::<(Entity, &GridPosition, &Wanted)>();
    for (entity, pos, wanted) in query.iter(world) {
        let dist = guard_pos.distance_chebyshev(pos) as f32;
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(entity);
        }
    }

    if let Some(target) = best_target {
        return Some((0.8, target)); // High priority
    }
    None
}

pub fn execute_arrest_system(
    world: &mut World,
    guard_entity: Entity,
    target_entity: Entity,
) {
    // 1. Remove Wanted
    world.entity_mut(target_entity).remove::<Wanted>();

    // 2. Add Inmate
    world.entity_mut(target_entity).insert(Inmate { sentence_ticks: 100 });

    // 3. Teleport to Jail (Find first Jail tile)
    let jail_pos = find_jail_spot(world);
    if let Some(pos) = jail_pos {
        *world.get_mut::<GridPosition>(target_entity).unwrap() = pos;
    }
}

fn find_jail_spot(world: &World) -> Option<GridPosition> {
    let zone_grid = world.resource::<ZoneGrid>();
    // Iterate grid to find Jail zone (inefficient for MVP but functional)
    // ...
    Some(GridPosition { x: 0, y: 0 }) // Stub
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: `find_jail_spot` scanning the whole grid is bad. Cache Jail tiles in `ZoneGrid` or a separate `JailTracker` resource.
- **Escort**: Instead of teleporting, the Guard should `PickUp` the prisoner (like Hauling) and carry them to the Jail.
- **Resistance**: Criminals should try to Flee or Fight back if they have high `Aggression`.
- **Prisoner Needs**: Inmates can't work. Guards must bring them Food (`ActionType::FeedInmate`).

## Acceptance Criteria

- [ ] `Wanted` component added to Vandalizing pops.
- [ ] `ActionType::Warden` added (COUNT = 17).
- [ ] GPU buffers updated correctly (alignment verified).
- [ ] `ZoneType::Jail` available.
- [ ] Guards successfully "arrest" (teleport) Wanted pops to Jail.
- [ ] Inmates are released after sentence.
- [ ] Tests pass.

## Technical Guidance

- **GPU Alignment**: `[u32; 17]` arrays + 6 floats + 2 ints = 172 bytes. Add `[u32; 1]` padding to hit 176 bytes (16-byte aligned).
- **Shader Update**: You MUST update `src/gpu/shaders/evaluate.wgsl` to match the new struct layout. If the shader uses fixed-size arrays, update them to 17.
