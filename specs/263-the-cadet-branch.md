# 263: The Cadet Branch

## Overview

"Babysitting the Emperor's nephew."

You can accept "Noble Scions" from your Home Faction (Layer 3). These Pops have the `Noble` trait.
- **Pros**: They provide a monthly **Allowance** (Credits) from their wealthy families as long as they are alive and happy.
- **Cons**: They refuse to work (Idle), have high Luxury needs, and often have negative traits like `Snob` or `Incompetent`.
- **Risk**: If a Scion dies, Relations with the Home Faction tank, leading to sanctions or war.

## Dependencies

- `003` — Pop Entity (Noble trait)
- `039` — Trade System (Credits)
- `031` — Pop Morale (Needs)
- `209` — Planetary Governance (Relations impact)

## RED Phase: Tests First

Write these tests in `src/layer1/social/cadet_branch_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Trait, Traits};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::social::cadet::{NobleScion, income_system, death_consequence_system};
    use crate::layer1::health::DeathEvent;
    use crate::layer2::governance::PlanetStats;

    #[test]
    fn test_noble_allowance_income() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Noble
        world.spawn((
            Pop,
            Traits(std::collections::HashSet::from([Trait::Noble])),
            NobleScion { allowance: 100.0 },
        ));

        // Run monthly tick (mocked)
        let mut schedule = Schedule::default();
        schedule.add_systems(income_system);
        schedule.run(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.credits, 100.0);
    }

    #[test]
    fn test_noble_refuses_work() {
        // This is a logic check in Job System, hard to test in isolation here unless we verify
        // the trait prevents assignment.
        // Assuming we add a helper `can_assign_job`
        let scion = NobleScion { allowance: 10.0 };
        // assert!(!can_assign_job(&scion, JobType::Miner));
    }

    #[test]
    fn test_noble_death_penalizes_relations() {
        let mut world = World::new();
        world.insert_resource(Events::<DeathEvent>::default());
        // Mock PlanetStats for relations
        world.insert_resource(PlanetStats { unrest_modifier: 0.0, ..Default::default() });
        // Or specific Relations resource

        let noble = world.spawn((
            Pop,
            NobleScion { allowance: 100.0 },
        )).id();

        // Kill them
        world.send_event(DeathEvent { entity: noble });

        let mut schedule = Schedule::default();
        schedule.add_systems(death_consequence_system);
        schedule.run(&mut world);

        // Check consequences
        // e.g. Unrest up, or specific Relation metric down
        // For GREEN, check PlanetStats modifier or resource
        let stats = world.resource::<PlanetStats>();
        assert!(stats.unrest_modifier > 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/social/cadet.rs

use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::health::DeathEvent;
use crate::layer2::governance::PlanetStats; // Using existing resource for global stats

#[derive(Component)]
pub struct NobleScion {
    pub allowance: f32,
}

pub fn income_system(
    mut resources: ResMut<ColonyResources>,
    query: Query<&NobleScion>,
    // Add Time resource to check for "Month" end
) {
    // Mock: assume run once per month
    for scion in query.iter() {
        resources.credits += scion.allowance;
    }
}

pub fn death_consequence_system(
    mut events: EventReader<DeathEvent>,
    query: Query<&NobleScion>,
    mut stats: ResMut<PlanetStats>,
) {
    for event in events.read() {
        if let Ok(_) = query.get(event.entity) {
            // Noble died!
            stats.unrest_modifier += 0.5; // Massive penalty
            // Log to Chronicle
        }
    }
}
```

### 2. Trait Update

Add `Noble` to `Trait` enum.
Modify `assign_job` logic in `009` to reject `Trait::Noble` for manual labor.

## REFACTOR Phase: Quality & Design

- **Happiness Scaling**: Allowance should scale with Happiness. Unhappy nobles write home complaining, reducing the payment.
- **Ransom**: If kidnapped by pirates, you must pay.
- **Events**: "The Scion wants a Pony." (Demands exotic pet or resource).

## Acceptance Criteria

- [ ] `NobleScion` component tracks income.
- [ ] System adds credits periodically.
- [ ] Death triggers penalty.
- [ ] Tests pass.

## Technical Guidance

- Use `ColonyResources` for credits.
- Hook into `DeathEvent` stream.
