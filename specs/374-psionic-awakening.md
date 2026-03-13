# 374: Psionic Awakening

## Overview

"Mind over matter, with terrible consequences."

**Psionic Awakening** introduces random Pops developing "Psionic Potential." They can perform tasks from a distance or soothe angry Pops. However, using these powers strains their "Sanity." Low sanity causes them to manifest psychic anomalies (fires, telekinetic outbursts) or go violently insane.

This creates tension: powerful, unique abilities vs. managing the fragile mental state of the gifted. Your best doctor might heal the entire colony during a plague without leaving their room, but the strain breaks their mind, and they start setting the hospital on fire with their thoughts.

## Dependencies

- `031` — Pop Morale (Sanity component)
- `016` — Utility AI System (For telekinetic tasks)

## RED Phase: Tests First

Write these tests in `src/layer1/traits/psionic_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Needs};
    use crate::layer1::traits::{PsionicPotential, Sanity, process_psionic_actions_system, trigger_psionic_breakdown_system};
    use crate::layer1::utility_types::{ActionType, PopAction};

    #[test]
    fn test_psionic_tasks_drain_sanity() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            PsionicPotential { level: 1 },
            Sanity { current: 100.0, max: 100.0 },
            PopAction { current: ActionType::TelekineticWork },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_psionic_actions_system);
        schedule.run(&mut world);

        // Using powers should drain sanity
        let sanity = world.get::<Sanity>(pop).unwrap();
        assert!(sanity.current < 100.0);
    }

    #[test]
    fn test_low_sanity_triggers_anomalies() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            PsionicPotential { level: 1 },
            Sanity { current: 0.0, max: 100.0 }, // Broken sanity
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(trigger_psionic_breakdown_system);
        schedule.run(&mut world);

        // Should trigger an anomaly event like fire or violence
        let events = world.resource::<Events<crate::layer1::hazards::FireEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some());
    }

    #[test]
    fn test_psion_soothes_nearby_pops() {
        let mut world = World::new();

        let psion = world.spawn((
            Pop,
            PsionicPotential { level: 1 },
            Sanity { current: 100.0, max: 100.0 },
            crate::layer1::map::GridPosition { x: 5, y: 5 },
        )).id();

        let angry_pop = world.spawn((
            Pop,
            Needs { leisure: 0.0, ..Default::default() }, // Needs soothing
            crate::layer1::map::GridPosition { x: 6, y: 5 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_psionic_soothing_system);
        schedule.run(&mut world);

        // Angry pop gets a temporary morale boost/leisure filled
        let needs = world.get::<Needs>(angry_pop).unwrap();
        assert!(needs.leisure > 0.0);

        // Psion loses some sanity
        let sanity = world.get::<Sanity>(psion).unwrap();
        assert!(sanity.current < 100.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/traits/psionic.rs

use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct PsionicPotential {
    pub level: u32,
}

#[derive(Component, Debug, Clone)]
pub struct Sanity {
    pub current: f32,
    pub max: f32,
}

impl Default for Sanity {
    fn default() -> Self {
        Self { current: 100.0, max: 100.0 }
    }
}
```

### 2. Systems

```rust
use crate::layer1::pop::{Pop, Needs};
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::layer1::map::GridPosition;
use crate::layer1::hazards::FireEvent;

pub fn process_psionic_actions_system(
    mut query: Query<(&mut Sanity, &PopAction), With<PsionicPotential>>,
) {
    for (mut sanity, action) in query.iter_mut() {
        if action.current == ActionType::TelekineticWork {
            sanity.current = (sanity.current - 5.0).max(0.0);
        }
    }
}

pub fn process_psionic_soothing_system(
    mut psions: Query<(&mut Sanity, &GridPosition), With<PsionicPotential>>,
    mut pops: Query<(&mut Needs, &GridPosition), Without<PsionicPotential>>,
) {
    for (mut sanity, psion_pos) in psions.iter_mut() {
        for (mut needs, pop_pos) in pops.iter_mut() {
            let dist = ((psion_pos.x - pop_pos.x).pow(2) + (psion_pos.y - pop_pos.y).pow(2)) as f32;
            if dist <= 4.0 && needs.leisure < 50.0 {
                // Soothe
                needs.leisure = (needs.leisure + 20.0).min(100.0);
                sanity.current = (sanity.current - 10.0).max(0.0);
            }
        }
    }
}

pub fn trigger_psionic_breakdown_system(
    query: Query<(&Sanity, &GridPosition), With<PsionicPotential>>,
    mut fire_events: EventWriter<FireEvent>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for (sanity, pos) in query.iter() {
        if sanity.current == 0.0 && rng.gen::<f32>() < 0.1 {
            fire_events.send(FireEvent { position: *pos });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Utility Action**: The `TelekineticWork` action should allow the Pop to progress tasks at a much greater distance without actually moving to the `TargetLocation`. This requires hooking into the pathfinding/job-execution logic to override distance checks if `PsionicPotential` is present.
- **Visuals**: Psionics acting or breaking down should emit unique particle effects or UI warnings.
- **Sanity Decay**: Integrate `Sanity` properly with existing Morale/Needs systems so stressful conditions lower it faster.

## Acceptance Criteria

- [ ] `PsionicPotential` and `Sanity` components exist.
- [ ] Psionic tasks like soothing nearby Pops increase `Needs::leisure` but drain `Sanity`.
- [ ] Reaching 0 Sanity occasionally triggers destructive events like `FireEvent`.
- [ ] Tests pass.

## Technical Guidance

- Ensure `Sanity` is distinct from standard `Morale` or `Needs::leisure` as it acts as mana/health for the psionics.
- Place `process_psionic_actions_system` in `Layer1SystemSet::Execution`.
- Ensure `ActionType::TelekineticWork` (or a similar psionic override flag) is defined in the `Utility AI` architecture.

## Questions

*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
