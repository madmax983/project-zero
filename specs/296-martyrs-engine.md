# 296: The Martyr's Engine

## Overview

"The ultimate sacrifice powers the ultimate machine."

The Martyr's Engine is an ancient precursor reactor that requires no fuel, but must be periodically "Attuned" by a Pop. The Pop is permanently consumed by the machine (killed/despawned). In exchange, the reactor generates infinite, clean energy and massive Layer 2 shielding for a full in-game year. However, the colony suffers a severe, long-lasting morale and stress penalty due to the sacrifice of a colonist.

- **Component**: `MartyrsEngine` (tracks attunement duration).
- **Action**: `AttuneEngineAction` (a Pop sacrifices themselves).
- **Consequence**: Massive `PowerSource` boost, `Shielding` boost, and a global `StressTracker` penalty to all remaining Pops.

## Dependencies

- `009` — Job System (for assigning the Attune action)
- `042` — Energy System (for `PowerSource` generation)
- `127` — Stress Breakdowns (for the colony-wide stress penalty)
- `152` — Orbital Stations / Layer 2 Shielding (for the shielding effect)

## RED Phase: Tests First

Write these tests in `src/layer1/tech/martyrs_engine_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::tech::martyrs_engine::{MartyrsEngine, process_martyrs_engine, AttuneEngineAction, attune_engine_system};
    use crate::layer1::power::{PowerSource, EnergyGrid};
    use crate::layer1::pop::{PopBundle, StressTracker};
    use crate::layer2::shielding::OrbitalShield;

    #[test]
    fn test_martyrs_engine_inactive_produces_nothing() {
        let mut world = World::new();
        let engine = world.spawn((
            MartyrsEngine { ticks_remaining: 0 },
            PowerSource { output: 0.0, ..Default::default() },
            OrbitalShield { capacity: 0.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_martyrs_engine);
        schedule.run(&mut world);

        let power = world.get::<PowerSource>(engine).unwrap();
        assert_eq!(power.output, 0.0);
    }

    #[test]
    fn test_attuning_engine_consumes_pop_and_activates() {
        let mut world = World::new();
        let engine = world.spawn((
            MartyrsEngine { ticks_remaining: 0 },
            PowerSource { output: 0.0, ..Default::default() },
            OrbitalShield { capacity: 0.0, ..Default::default() },
        )).id();

        let pop = world.spawn(PopBundle::default()).id();

        world.spawn(AttuneEngineAction {
            target_engine: engine,
            pop,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(attune_engine_system);
        schedule.run(&mut world);

        // Pop is consumed
        assert!(world.get_entity(pop).is_err() || world.get_entity(pop).unwrap().is_despawned());

        // Engine is active
        let engine_comp = world.get::<MartyrsEngine>(engine).unwrap();
        assert!(engine_comp.ticks_remaining > 0);
    }

    #[test]
    fn test_active_engine_produces_power_and_decays() {
        let mut world = World::new();
        let engine = world.spawn((
            MartyrsEngine { ticks_remaining: 10 },
            PowerSource { output: 0.0, ..Default::default() },
            OrbitalShield { capacity: 0.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_martyrs_engine);
        schedule.run(&mut world);

        let engine_comp = world.get::<MartyrsEngine>(engine).unwrap();
        assert_eq!(engine_comp.ticks_remaining, 9); // Decays

        let power = world.get::<PowerSource>(engine).unwrap();
        assert!(power.output > 0.0); // Producing power
    }

    #[test]
    fn test_attuning_causes_colony_stress() {
        let mut world = World::new();
        let engine = world.spawn(MartyrsEngine { ticks_remaining: 0 }).id();
        let pop = world.spawn(PopBundle::default()).id();
        let bystander = world.spawn(StressTracker { accumulated_stress: 0.0, ..Default::default() }).id();

        world.spawn(AttuneEngineAction { target_engine: engine, pop });

        let mut schedule = Schedule::default();
        schedule.add_systems(attune_engine_system);
        schedule.run(&mut world);

        let bystander_stress = world.get::<StressTracker>(bystander).unwrap();
        assert!(bystander_stress.accumulated_stress > 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components & Actions

```rust
// src/layer1/tech/martyrs_engine.rs

use bevy_ecs::prelude::*;
use crate::layer1::power::PowerSource;
use crate::layer1::pop::StressTracker;
use crate::layer2::shielding::OrbitalShield;

#[derive(Component)]
pub struct MartyrsEngine {
    pub ticks_remaining: u32,
}

#[derive(Component)]
pub struct AttuneEngineAction {
    pub target_engine: Entity,
    pub pop: Entity,
}

// Constants
const ATTUNE_DURATION_TICKS: u32 = 1000; // E.g., one year
const ENGINE_POWER_OUTPUT: f32 = 10000.0;
const ENGINE_SHIELD_CAPACITY: f32 = 5000.0;
const SACRIFICE_STRESS_PENALTY: f32 = 40.0;

pub fn attune_engine_system(
    mut commands: Commands,
    action_query: Query<(Entity, &AttuneEngineAction)>,
    mut engine_query: Query<&mut MartyrsEngine>,
    mut stress_query: Query<&mut StressTracker>,
) {
    for (action_entity, action) in action_query.iter() {
        // Activate Engine
        if let Ok(mut engine) = engine_query.get_mut(action.target_engine) {
            engine.ticks_remaining = ATTUNE_DURATION_TICKS;
        }

        // Consume Pop
        commands.entity(action.pop).despawn_recursive();

        // Apply Stress to everyone else
        for mut stress in stress_query.iter_mut() {
            stress.accumulated_stress += SACRIFICE_STRESS_PENALTY;
        }

        // Cleanup Action
        commands.entity(action_entity).despawn();
    }
}

pub fn process_martyrs_engine(
    mut query: Query<(&mut MartyrsEngine, &mut PowerSource, &mut OrbitalShield)>,
) {
    for (mut engine, mut power, mut shield) in query.iter_mut() {
        if engine.ticks_remaining > 0 {
            engine.ticks_remaining -= 1;
            power.output = ENGINE_POWER_OUTPUT;
            shield.capacity = ENGINE_SHIELD_CAPACITY;
        } else {
            power.output = 0.0;
            shield.capacity = 0.0;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **VFX & Chronicle**: Add a `ChronicleEvent` documenting the sacrifice. Hook up a massive, eerie glow to the rendering system when `ticks_remaining > 0`.
- **Target Selection**: The `AttuneEngineAction` should ideally be performed by a high-skill or specific Pop, maybe scaling the `ticks_remaining` based on the sacrificed Pop's level or specific traits.
- **Stress Application**: Instead of instantly applying stress to everyone, maybe emit a `ColonyMoraleEvent` that is processed gracefully by the `social` or `culture` modules, to integrate with existing traits like `Callous` or `Fringe`.

## Acceptance Criteria

- [ ] `MartyrsEngine` produces massive power and shielding when active.
- [ ] `MartyrsEngine` decays per tick down to 0, stopping output.
- [ ] `AttuneEngineAction` successfully despawns a Pop.
- [ ] `AttuneEngineAction` applies a significant stress penalty to all other Pops.
- [ ] All tests pass with zero failures.

## Technical Guidance

- Since `Pop` destruction is absolute, make sure you don't leave dangling references in Job or Faction modules. Despawning recursively usually triggers cleanup through Bevy's hierarchy, but verify logic.
- Ensure `OrbitalShield` defaults exist or adjust layer dependencies if Layer 2 shielding is not yet fully merged.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
