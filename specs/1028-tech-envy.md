# 1028: Tech Envy

## 1. Overview
If a high-tier machine (e.g., "Tier 2") is constructed in the colony, Pops assigned to work on lower-tier machines of the same category suffer an "Obsolescence" mood penalty. This forces players to either upgrade all facilities simultaneously or manage targeted strikes and resentment from the workforce using the old gear.

## 2. Dependencies
- Layer 1 `Buildings`/`Machines` (Tier levels and categories).
- Layer 1 `Pop` entity and `WorkOrder`/`Job` assignments.
- Layer 1 `Mood` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::buildings::{Machine, TechTier};
    use crate::layer1::pop::{Pop, Mood};
    use crate::layer1::utility_ai::WorkOrder;

    #[test]
    fn test_working_on_low_tier_machine_causes_envy_when_high_tier_exists() {
        let mut app = App::new();
        app.add_systems(Update, apply_tech_envy_system);

        // High Tier Machine
        app.world_mut().spawn((
            Machine { category: "Fabricator".to_string() },
            TechTier { level: 2 },
        ));

        // Low Tier Machine
        let old_machine = app.world_mut().spawn((
            Machine { category: "Fabricator".to_string() },
            TechTier { level: 1 },
        )).id();

        let jealous_worker = app.world_mut().spawn((
            Pop,
            Mood { value: 100.0 },
            WorkOrder { task_type: "Fabricate".to_string(), target: Some(old_machine), ..default() }
        )).id();

        app.update();

        let mood = app.world().get::<Mood>(jealous_worker).unwrap();
        assert!(mood.value < 100.0, "Working on a Tier 1 machine when a Tier 2 exists should cause a mood penalty.");
    }

    #[test]
    fn test_no_envy_if_highest_tier_machine_used() {
        let mut app = App::new();
        app.add_systems(Update, apply_tech_envy_system);

        let new_machine = app.world_mut().spawn((
            Machine { category: "Fabricator".to_string() },
            TechTier { level: 2 },
        )).id();

        // Also spawn an old one so the high tier is strictly higher
        app.world_mut().spawn((
            Machine { category: "Fabricator".to_string() },
            TechTier { level: 1 },
        ));

        let happy_worker = app.world_mut().spawn((
            Pop,
            Mood { value: 100.0 },
            WorkOrder { task_type: "Fabricate".to_string(), target: Some(new_machine), ..default() }
        )).id();

        app.update();

        let mood = app.world().get::<Mood>(happy_worker).unwrap();
        assert_eq!(mood.value, 100.0, "Working on the highest tier machine should not cause an envy penalty.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/tech_envy.rs
use bevy::prelude::*;
use crate::layer1::buildings::{Machine, TechTier};
use crate::layer1::pop::{Pop, Mood};
use crate::layer1::utility_ai::WorkOrder;
use std::collections::HashMap;

pub fn apply_tech_envy_system(
    machine_query: Query<(&Machine, &TechTier)>,
    mut worker_query: Query<(&mut Mood, &WorkOrder), With<Pop>>,
) {
    // Determine the max tier for each category
    let mut max_tiers: HashMap<String, u32> = HashMap::new();
    for (machine, tier) in machine_query.iter() {
        let current_max = max_tiers.entry(machine.category.clone()).or_insert(0);
        if tier.level > *current_max {
            *current_max = tier.level;
        }
    }

    // Apply penalty to workers on sub-optimal machines
    for (mut mood, order) in worker_query.iter_mut() {
        if let Some(target) = order.target {
            if let Ok((machine, tier)) = machine_query.get(target) {
                if let Some(max_tier) = max_tiers.get(&machine.category) {
                    if tier.level < *max_tier {
                        // MVP: Flat penalty per tick they are working on the inferior machine
                        let difference = *max_tier - tier.level;
                        mood.value -= (difference as f32) * 0.1;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Strike Mechanics:** When Mood hits a critical threshold due specifically to this penalty, it should trigger a specific `StrikeEvent` targeting that machine category, preventing further work until upgraded.
- **Line of Sight/Awareness:** Does the worker need to *know* the better machine exists? Maybe the penalty only applies if the machines are in the same `Zone` or if the Pop is high enough `Status` to care.
- **Performance:** Building the `max_tiers` HashMap every frame is inefficient if machines aren't being built/destroyed constantly. Cache this in a Resource and only update on `MachineSpawnEvent` / `MachineDestroyEvent`.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_working_on_low_tier_machine_causes_envy_when_high_tier_exists` passes.
- [ ] Test `test_no_envy_if_highest_tier_machine_used` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Integration with the `Utility AI` is minimal; this is just a modifier to `Mood` based on the current assigned `WorkOrder`.
- Ensure `Machine` categories are normalized (e.g., enums rather than strings) to prevent typos breaking the max-tier logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
