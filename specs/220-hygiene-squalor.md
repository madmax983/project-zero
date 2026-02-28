# 220: Hygiene & Squalor

## Overview

A clean ship is a happy ship. Space is dirty, and disease breeds in filth.

This feature introduces a **Hygiene** need and a **Filth** mechanic. Pops accumulate Filth over time, accelerated by "dirty" jobs (Mining, Butchery). High Filth lowers Hygiene, increases `DiseaseRisk`, and causes social penalties ("Repulsive").

Players must build **Showers** to allow Pops to clean themselves. Showers consume Water.

## Dependencies

- `005` — Pop Needs (Hygiene as a new need)
- `034` — Pop Health (Disease risk factor)
- `009` — Job System (Dirty jobs)

## RED Phase: Tests First

Write these tests in `src/layer1/hygiene_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, PopState};
    use crate::layer1::needs::Needs;
    use crate::layer1::hygiene::{Filth, filth_accumulation_system, hygiene_decay_system, shower_use_system};
    use crate::layer1::job::Job;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::building::{Building, BuildingType};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            water: 100.0,
            ..Default::default()
        });
        world
    }

    #[test]
    fn test_filth_accumulation_idle() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop::default(),
            PopState::Idle,
            Filth { current: 0.0, max: 100.0 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(filth_accumulation_system);
        schedule.run(&mut world);

        let filth = world.get::<Filth>(pop).unwrap();
        assert!(filth.current > 0.0, "Filth should accumulate slowly even when idle");
    }

    #[test]
    fn test_filth_accumulation_dirty_job() {
        let mut world = setup_world();
        let miner = world.spawn((
            Pop::default(),
            PopState::Working,
            Job::Miner, // Assumed dirty
            Filth { current: 0.0, max: 100.0 },
        )).id();

        let researcher = world.spawn((
            Pop::default(),
            PopState::Working,
            Job::Researcher, // Assumed clean
            Filth { current: 0.0, max: 100.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(filth_accumulation_system);
        schedule.run(&mut world);

        let miner_filth = world.get::<Filth>(miner).unwrap().current;
        let researcher_filth = world.get::<Filth>(researcher).unwrap().current;

        assert!(miner_filth > researcher_filth, "Miners should get dirtier than Researchers");
    }

    #[test]
    fn test_hygiene_decay_from_filth() {
        let mut world = setup_world();
        let clean_pop = world.spawn((
            Needs { hygiene: 100.0, ..Default::default() },
            Filth { current: 0.0, ..Default::default() },
        )).id();

        let dirty_pop = world.spawn((
            Needs { hygiene: 100.0, ..Default::default() },
            Filth { current: 100.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hygiene_decay_system);
        schedule.run(&mut world);

        let clean_hygiene = world.get::<Needs>(clean_pop).unwrap().hygiene;
        let dirty_hygiene = world.get::<Needs>(dirty_pop).unwrap().hygiene;

        assert!(dirty_hygiene < clean_hygiene, "High filth should accelerate hygiene decay");
    }

    #[test]
    fn test_shower_usage() {
        let mut world = setup_world();

        let pop = world.spawn((
            Needs { hygiene: 10.0, ..Default::default() },
            Filth { current: 80.0, ..Default::default() },
            // In a real scenario, Utility AI would drive this.
            // Here we test the action effect directly or a system that detects "UsingShower" state.
            crate::layer1::actions::CurrentAction {
                action_type: crate::layer1::actions::ActionType::UseShower,
                target: Some(Entity::from_raw(1)), // The shower
                ..Default::default()
            },
        )).id();

        let shower = world.spawn((
            Building { building_type: BuildingType::Shower },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(shower_use_system);
        schedule.run(&mut world);

        // Check Water Consumption
        let resources = world.resource::<ColonyResources>();
        assert!(resources.water < 100.0, "Shower should consume water");

        // Check Hygiene/Filth
        let needs = world.get::<Needs>(pop).unwrap();
        let filth = world.get::<Filth>(pop).unwrap();

        assert!(needs.hygiene > 10.0, "Hygiene should increase");
        assert!(filth.current < 80.0, "Filth should decrease");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `Needs` Struct

In `src/layer1/needs.rs`:

```rust
#[derive(Component, Debug, Clone, Default)]
pub struct Needs {
    pub hunger: f32,
    pub rest: f32,
    pub social: f32,
    pub hygiene: f32, // New field
}
```

### 2. Define `Filth` & `Hygiene` Logic

Create `src/layer1/hygiene.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::pop::{PopState, Pop};
use crate::layer1::job::Job;
use crate::layer1::needs::Needs;
use crate::layer1::resources::ColonyResources;
use crate::layer1::actions::{CurrentAction, ActionType};

#[derive(Component, Debug, Clone, Default)]
pub struct Filth {
    pub current: f32,
    pub max: f32, // Usually 100.0
}

pub const BASE_FILTH_RATE: f32 = 0.1;
pub const DIRTY_JOB_MULTIPLIER: f32 = 3.0;
pub const CLEAN_JOB_MULTIPLIER: f32 = 0.5;
pub const SHOWER_WATER_COST: f32 = 5.0;
pub const HYGIENE_RESTORE_RATE: f32 = 10.0;
pub const FILTH_CLEAN_RATE: f32 = 20.0;

pub fn filth_accumulation_system(mut query: Query<(&mut Filth, &PopState, Option<&Job>)>) {
    for (mut filth, state, job) in query.iter_mut() {
        let rate = match state {
            PopState::Working => {
                if let Some(j) = job {
                    match j {
                        Job::Miner | Job::Hauler => BASE_FILTH_RATE * DIRTY_JOB_MULTIPLIER,
                        Job::Researcher | Job::Doctor => BASE_FILTH_RATE * CLEAN_JOB_MULTIPLIER,
                        _ => BASE_FILTH_RATE,
                    }
                } else {
                    BASE_FILTH_RATE
                }
            }
            _ => BASE_FILTH_RATE * 0.2, // Idle accumulation
        };
        filth.current = (filth.current + rate).min(filth.max);
    }
}

pub fn hygiene_decay_system(mut query: Query<(&mut Needs, &Filth)>) {
    for (mut needs, filth) in query.iter_mut() {
        // Filth accelerates hygiene loss
        let decay = 0.05 + (filth.current / 100.0) * 0.1;
        needs.hygiene = (needs.hygiene - decay).max(0.0);
    }
}

pub fn shower_use_system(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(Entity, &mut Needs, &mut Filth, &CurrentAction)>,
) {
    // Basic implementation: if action is UseShower, consume water and clean
    if resources.water < 0.1 { return; } // No water, no shower

    for (entity, mut needs, mut filth, action) in query.iter_mut() {
        if action.action_type == ActionType::UseShower {
            // Consume water (per tick or per action? For TDD simplicity, assume per tick)
            // Ideally this is handled in `actions/mod.rs`, but for Green phase here is ok
            resources.consume(crate::layer1::resources::ResourceType::Water, 0.1); // Need to add Water to Enum or use field directly
            // Oh wait, ColonyResources.consume handles types. We need to add Water to ItemType enum or handle it manually.
            // For now:
            resources.water -= 0.1;

            needs.hygiene = (needs.hygiene + HYGIENE_RESTORE_RATE).min(100.0);
            filth.current = (filth.current - FILTH_CLEAN_RATE).max(0.0);

            // If clean, end action (logic usually in action system)
        }
    }
}
```

### 3. Add `Shower` Building

In `src/layer1/building.rs`, add `Shower` to `BuildingType`.

### 4. Update `ActionType`

In `src/layer1/actions.rs`, add `UseShower`.

## REFACTOR Phase: Quality & Design

- **ResourceType::Water**: Add `Water` to `ResourceType` enum to allow standard `consume` usage.
- **Disease Risk**: Integrate `Filth` into the `calculate_disease_risk` function in `health.rs`.
- **Social**: Add a system that checks `Filth > 80` and applies a temporary `Trait::Repulsive` or just a social interaction penalty.
- **Visuals**: Add a "dirt" overlay sprite to Pops with high filth.
- **Utility AI**: Add `UseShower` scorer to `utility_ai.rs` based on `(100 - hygiene) + filth`.

## Acceptance Criteria

- [ ] `Filth` accumulates based on job type.
- [ ] `Hygiene` need exists and decays faster with high filth.
- [ ] `Shower` building can be built.
- [ ] Pops use showers to clean filth and restore hygiene.
- [ ] Showers consume Water.
- [ ] Water resource is correctly decremented.

## Technical Guidance

- Use `ColonyResources.water` directly until `ResourceType::Water` refactor is done.
- Ensure `Shower` building has `WaterConsumer` tag if we implement pipe grids later.
- Don't forget to register `Hygiene` and `Filth` components in `lib.rs` or `main.rs`.

## Questions

- *Builder: Should Showers require Power?*
*Architect: For now, no. Assume they are gravity fed from water reserves.*
    - *Architect: Yes, keep showers power-free (gravity-fed) for the MVP.*
- *Builder: Does Filth affect Room Quality?*
*Architect: Yes, dirty pops standing in a room should lower its beauty/cleanliness score temporarily.*
  - *Architect: Yes, filth temporarily reduces Room Quality.*
- *Architect: Yes, filth temporarily reduces Room Quality.*
    - *Architect: Agreed. Filth should temporarily reduce the Room Quality where dirty pops are standing.*
