# 1026: The Lottery

## 1. Overview
When resources fall to critical levels, the colony can enact "The Lottery" to sacrifice a portion of the population (either by exile or execution) to save the rest. While this immediately relieves resource strain, the surviving population suffers immense "Survivor's Guilt" (trauma/mood penalty), creating a grim choice between collective starvation and intentional sacrifice.

## 2. Dependencies
- Layer 1 `Economy` (Resource monitoring).
- Layer 1 `Pop` entity (Death/Exile mechanics).
- Layer 1 `Edict`/`Policy` system to enact it.
- Layer 1 `Mood`/`Memory` system (Survivor's Guilt).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::economy::RawResources;
    use crate::layer1::pop::{Pop, DeathEvent};
    use crate::layer1::mood::{Mood, TraumaEvent};
    use crate::layer1::policies::EdictEvent;

    #[test]
    fn test_lottery_edict_sacrifices_pops() {
        let mut app = App::new();
        app.add_event::<EdictEvent>();
        app.add_event::<DeathEvent>();
        app.add_event::<TraumaEvent>(); // Ensure this is registered
        app.add_systems(Update, execute_lottery_system);

        let pop1 = app.world_mut().spawn((Pop, Mood { value: 100.0 })).id();
        let pop2 = app.world_mut().spawn((Pop, Mood { value: 100.0 })).id();

        app.world_mut().resource_mut::<Events<EdictEvent>>().send(EdictEvent {
            edict_type: "TheLottery".to_string(),
            target_count: 1, // Sacrifice 1 pop
        });

        app.update();

        // Verify one pop died
        let death_events = app.world().resource::<Events<DeathEvent>>();
        assert_eq!(death_events.get_reader().len(death_events), 1, "The Lottery should trigger a DeathEvent for the targeted count.");
    }

    #[test]
    fn test_surviving_pops_receive_survivors_guilt_trauma() {
        let mut app = App::new();
        app.add_event::<EdictEvent>();
        app.add_event::<DeathEvent>();
        app.add_event::<TraumaEvent>();
        app.add_systems(Update, execute_lottery_system);

        let survivor = app.world_mut().spawn((Pop, Mood { value: 100.0 })).id();

        app.world_mut().resource_mut::<Events<EdictEvent>>().send(EdictEvent {
            edict_type: "TheLottery".to_string(),
            target_count: 0, // We just want to test the trauma application on the rest
        });

        // Wait, for the test to work, the system needs to process a sacrifice to apply trauma.
        // Let's spawn a sacrificial lamb.
        let lamb = app.world_mut().spawn((Pop, Mood { value: 100.0 })).id();

        app.world_mut().resource_mut::<Events<EdictEvent>>().send(EdictEvent {
            edict_type: "TheLottery".to_string(),
            target_count: 1,
        });

        app.update();

        let trauma_events = app.world().resource::<Events<TraumaEvent>>();
        let mut reader = trauma_events.get_reader();
        let mut found_trauma = false;
        for event in reader.read(trauma_events) {
            if event.pop == survivor && event.trauma_type == "SurvivorsGuilt" {
                found_trauma = true;
            }
        }

        assert!(found_trauma, "Surviving Pops should receive the Survivor's Guilt trauma.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/the_lottery.rs
use bevy::prelude::*;
use crate::layer1::pop::{Pop, DeathEvent};
use crate::layer1::mood::{Mood, TraumaEvent};
use crate::layer1::policies::EdictEvent;

pub fn execute_lottery_system(
    mut edict_events: EventReader<EdictEvent>,
    pop_query: Query<Entity, With<Pop>>,
    mut death_events: EventWriter<DeathEvent>,
    mut trauma_events: EventWriter<TraumaEvent>,
) {
    for event in edict_events.read() {
        if event.edict_type == "TheLottery" {
            let mut all_pops: Vec<Entity> = pop_query.iter().collect();

            // Very simple random selection (using first N for MVP)
            let sacrifice_count = std::cmp::min(event.target_count as usize, all_pops.len());

            for i in 0..sacrifice_count {
                let victim = all_pops[i];
                death_events.send(DeathEvent {
                    pop: victim,
                    cause: "The Lottery".to_string(),
                });
            }

            // Apply trauma to survivors
            for i in sacrifice_count..all_pops.len() {
                let survivor = all_pops[i];
                trauma_events.send(TraumaEvent {
                    pop: survivor,
                    trauma_type: "SurvivorsGuilt".to_string(),
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Randomization:** The MVP sacrifices the first `N` entities returned by the query. It must use an RNG to be an actual lottery.
- **Selection Bias:** Players might want to "rig" the lottery to target low-skill Pops, Criminals, or the Elderly, introducing political consequences.
- **UI Trigger:** The Edict should only be selectable if `Food` or `Water` resources are projected to hit 0 within X days.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_lottery_edict_sacrifices_pops` passes.
- [ ] Test `test_surviving_pops_receive_survivors_guilt_trauma` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Ensure `TraumaEvent` translates into a long-term `Mood` penalty in the mood processing system.
- `EdictEvent` should probably carry a `target_count` or percentage parameter defined by the player in the UI.

## 8. Questions
*Builder: add questions here if spec is unclear.*
