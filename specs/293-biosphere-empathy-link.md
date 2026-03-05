# Specification: The Biosphere Empathy Link (Layer 1)

## 1. Overview
The planet is alive, and your people are becoming a part of its nervous system. Pops living in close proximity to native "Empathic Flora" slowly gain a shared "Hive-Mind" trait. Their needs and moods synchronize, and they gain massive efficiency bonuses when working together. However, damaging the flora globally injures them or causes massive stress spikes, making the colony terrifyingly efficient but extremely vulnerable to ecological damage.

## 2. Dependencies
- `044` Horticulture & Beauty (for flora).
- `084` Pop Traits (for Hive-Mind trait).
- `031` Pop Morale (for mood sync).

## 3. RED Phase: Tests First

```rust
// src/layer1/biosphere_empathy_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Trait, Traits};
    use crate::layer1::needs::StressTracker;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(GlobalFloraHealth { total_health: 100.0 });
        world
    }

    #[test]
    fn test_empathic_pops_sync_stress() {
        let mut world = setup_world();

        let pop1 = world.spawn((
            Pop,
            Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
            StressTracker { accumulated_stress: 80.0, ..Default::default() },
        )).id();

        let pop2 = world.spawn((
            Pop,
            Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
            StressTracker { accumulated_stress: 20.0, ..Default::default() },
        )).id();

        world.run_system_once(sync_empathic_network_system);

        // Stress should equalize towards the average (50)
        let s1 = world.get::<StressTracker>(pop1).unwrap().accumulated_stress;
        let s2 = world.get::<StressTracker>(pop2).unwrap().accumulated_stress;

        assert!((s1 - 50.0).abs() < 10.0);
        assert!((s2 - 50.0).abs() < 10.0);
    }

    #[test]
    fn test_flora_damage_spikes_stress() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
            StressTracker { accumulated_stress: 10.0, ..Default::default() },
        )).id();

        // Simulate global flora damage event
        world.send_event(FloraDamagedEvent { damage_amount: 50.0 });

        world.run_system_once(handle_flora_damage_empathy_system);

        let stress = world.get::<StressTracker>(pop).unwrap().accumulated_stress;
        // Stress should spike heavily
        assert!(stress > 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/environment/biosphere_empathy.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Trait, Traits};
use crate::layer1::needs::StressTracker;

#[derive(Resource, Default)]
pub struct GlobalFloraHealth {
    pub total_health: f32,
}

#[derive(Event)]
pub struct FloraDamagedEvent {
    pub damage_amount: f32,
}

pub fn sync_empathic_network_system(
    mut query: Query<(&Traits, &mut StressTracker)>,
) {
    let mut total_stress = 0.0;
    let mut count = 0;

    // Calculate average
    for (traits, stress) in query.iter() {
        if traits.has(Trait::EmpathicLink) {
            total_stress += stress.accumulated_stress;
            count += 1;
        }
    }

    if count == 0 { return; }
    let average_stress = total_stress / count as f32;

    // Apply pull towards average
    for (traits, mut stress) in query.iter_mut() {
        if traits.has(Trait::EmpathicLink) {
            // Pull 10% towards the average per tick
            stress.accumulated_stress += (average_stress - stress.accumulated_stress) * 0.1;
        }
    }
}

pub fn handle_flora_damage_empathy_system(
    mut events: EventReader<FloraDamagedEvent>,
    mut query: Query<(&Traits, &mut StressTracker)>,
) {
    let mut total_damage = 0.0;
    for event in events.read() {
        total_damage += event.damage_amount;
    }

    if total_damage > 0.0 {
        for (traits, mut stress) in query.iter_mut() {
            if traits.has(Trait::EmpathicLink) {
                // Flat stress penalty based on flora damage
                stress.accumulated_stress += total_damage * 0.5;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Acquisition:** Implement a system `update_flora_exposure_system` that tracks how long a Pop spends near `EmpathicFlora` objects and eventually grants them the `Trait::EmpathicLink`.
- **Efficiency Buff:** In `calculate_work_amount`, check if multiple `EmpathicLink` Pops are working adjacent to each other. If so, multiply their output.
- **UI:** The system view should warn the player before they bulldoze a forest if they have Empathic Pops.

## 6. Acceptance Criteria

- [ ] All tests pass.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Pops with `EmpathicLink` average their stress levels over time.
- [ ] Global damage to flora triggers a massive stress spike in linked Pops.

## 7. Technical Guidance
- **Trait Definition:** Add `EmpathicLink` to the `Trait` enum in `src/layer1/pop/traits.rs`.
- **Performance:** `sync_empathic_network_system` does two passes over the query. This is acceptable for MVP, but consider caching the average if it runs hot.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect: I will answer your questions as they come up.*
