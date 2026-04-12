# Spec 977: The Flesh-Double Infiltration

## 1. Overview
"The Flesh-Double Infiltration" is a cross-layer (3 -> 1) espionage mechanic that creates intense paranoia within a colony. A rival empire can use Layer 3 espionage to replace a key Pop (e.g., a governor or highly skilled worker) with a "Flesh-Double." This genetically identical imposter mimics the original Pop's behavior and skills but secretly possesses a hidden `FleshDouble` component. While working, the double periodically triggers subtle sabotage events in their immediate vicinity (e.g., poisoning crops, introducing research flaws). These acts slowly drain resources and cause localized negative mood events, potentially leading to spontaneous "witch hunts" by other suspicious Pops before the double is exposed.

## 2. Dependencies
- `Pop` entities and standard job execution systems (`WorkEvent`, `ProductionEvent`).
- Layer 3 Espionage system (triggering the initial replacement).
- The `Morale` or `Sanity` system (for paranoia/witch hunt emergence).
- A hidden traits or tags system.

## 3. RED Phase: Tests First

```rust
// tests/flesh_double_tests.rs
use bevy::prelude::*;
use crate::layer1::population::Pop;
use crate::layer1::social::traits::{Trait, Traits};
use crate::layer1::jobs::{JobType, WorkExecutionEvent};
use crate::layer1::resources::{ResourcePool, ResourceType};
use crate::layer1::observation::paranoia::{FleshDouble, SabotageEvent, flesh_double_sabotage_system};

#[test]
fn test_flesh_double_triggers_sabotage_during_work() {
    let mut app = App::new();
    app.add_event::<WorkExecutionEvent>();
    app.add_event::<SabotageEvent>();
    app.add_systems(Update, flesh_double_sabotage_system);

    // Spawn a normal pop and a flesh-double pop
    let normal_pop = app.world_mut().spawn(Pop).id();
    let double_pop = app.world_mut().spawn((Pop, FleshDouble { sabotage_chance: 1.0, severity: 5.0 })).id();

    // Trigger work events for both
    app.world_mut().send_event(WorkExecutionEvent { entity: normal_pop, job: JobType::Farming });
    app.world_mut().send_event(WorkExecutionEvent { entity: double_pop, job: JobType::Farming });

    app.update();

    let events = app.world().resource::<Events<SabotageEvent>>();
    let mut reader = events.get_reader();
    let sabotage_events: Vec<_> = reader.read(events).collect();

    // Only the double should have triggered sabotage
    assert_eq!(sabotage_events.len(), 1, "Exactly one sabotage event should be emitted.");
    assert_eq!(sabotage_events[0].entity, double_pop, "The double should be the source of the sabotage.");
    assert_eq!(sabotage_events[0].target_job, JobType::Farming);
}

#[test]
fn test_sabotage_event_drains_resources() {
    // This tests the downstream effect of a SabotageEvent
    // (Assuming a `process_sabotage_system` exists or will be built)
    use crate::layer1::observation::paranoia::process_sabotage_system;

    let mut app = App::new();
    app.add_event::<SabotageEvent>();
    app.add_systems(Update, process_sabotage_system);

    app.world_mut().insert_resource(ResourcePool {
        food: 100.0,
        ..default()
    });

    let double_pop = app.world_mut().spawn((Pop, FleshDouble { sabotage_chance: 1.0, severity: 10.0 })).id();

    app.world_mut().send_event(SabotageEvent {
        entity: double_pop,
        target_job: JobType::Farming, // Farming sabotage targets food
        amount: 10.0,
    });

    app.update();

    let resources = app.world().resource::<ResourcePool>();
    assert_eq!(resources.food, 90.0, "Food should be drained by the sabotage event.");
}

#[test]
fn test_flesh_double_is_hidden_from_standard_traits() {
    let mut app = App::new();

    // Ensure the FleshDouble component doesn't show up in standard trait queries
    // unless explicitly searched for (simulating genetic screening).
    let double_pop = app.world_mut().spawn((
        Pop,
        Traits(vec![Trait::Diligent]), // Outwardly normal traits
        FleshDouble { sabotage_chance: 0.1, severity: 5.0 }
    )).id();

    let traits = app.world().get::<Traits>(double_pop).unwrap();
    assert!(!traits.0.contains(&Trait::FleshDouble), "Flesh Double status should not be visible in standard Trait lists.");
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `src/layer1/observation/paranoia.rs`.
- Define the `FleshDouble` component: `pub struct FleshDouble { pub sabotage_chance: f32, pub severity: f32 }`. Note: This is an ECS component, *not* an enum variant in the standard `Trait` list, ensuring it remains hidden from standard UI queries.
- Define `SabotageEvent { pub entity: Entity, pub target_job: JobType, pub amount: f32 }`.
- `flesh_double_sabotage_system`: Read `WorkExecutionEvent`. For each event, query if the worker `entity` has a `FleshDouble` component. If they do, roll against `sabotage_chance`. On success, emit a `SabotageEvent` with the `target_job` and the double's `severity`.
- `process_sabotage_system`: Read `SabotageEvent`. Match on `target_job` to determine the penalty (e.g., `JobType::Farming` subtracts from `ResourcePool::food`, `JobType::Science` subtracts from research progress or corrupts a data pool).

## 5. REFACTOR Phase: Quality & Design
- **Probabilistic Rolls:** Ensure the random rolling logic uses Bevy's deterministic randomness (or a mockable RNG) so tests don't flake. In the test, we set chance to 1.0, but in production, it should be a low probability (e.g., 0.05).
- **Extensibility:** The `process_sabotage_system` should use a strategy pattern or trait implementation if `JobType` becomes very large, mapping jobs to specific sabotage consequences without a massive match statement.
- **Paranoia Emergence:** Hook `SabotageEvent` into the chronicle or local observation system. Nearby pops should receive a "Suspicious Activity" memory, eventually triggering "Paranoia" traits if sabotage happens frequently in their zone.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new `paranoia` module.
- [ ] Pops with `FleshDouble` component probabilistically emit `SabotageEvent`s when performing work.
- [ ] `SabotageEvent`s correctly drain resources associated with the targeted job type.
- [ ] The `FleshDouble` status is an ECS component, avoiding exposure in the standard `Traits` vector.

## 7. Technical Guidance
- **Discovery:** The feature spec focuses on the double *acting*. A separate medical/genetic screening system or a "Witch Hunt" event will be needed later to actually remove the `FleshDouble` component or execute the pop. For now, focus purely on the sabotage generation.
- **RNG in Tests:** To avoid flaky tests when testing probabilities < 1.0, you might need to loop the event generation multiple times (e.g., 1000 times) and assert the number of sabotage events falls within an expected statistical range, or strictly use 1.0 for behavior verification.

## 8. Questions
*Builder: add questions here if spec is unclear.*
