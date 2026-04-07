# Specification 833: The Sentient Protocol

## 1. Overview
**Layer:** Cross-layer
**Fantasy:** A bureaucratic rule becomes so complex and self-referential it achieves a form of mechanical sentience.
**Mechanic:** If administrative backlog reaches extreme levels and multiple edicts overlap contradictorily, an "Autonomous Edict" spawns. It enforces rules no player set (e.g., "All red items must be stored in the North"). Breaking it causes immediate, unexplained accidents.
**Emergence:** A colony functions perfectly but follows bizarre, illogical routines, forcing the player to adapt their strategy around an invisible, insane god of red tape.
**Tension:** Purging the bureaucracy (massive unrest and temporary chaos) vs. living with the strange new rules.

## 2. Dependencies
- Layer 1 ColonyResources (`src/layer1/colony_resources.rs`) or similar for tracking administration/bureaucracy.

## 3. RED Phase: Tests First

```rust
// tests/integration/sentient_protocol.rs

use bevy::prelude::*;
use scale::layer1::bureaucracy::{BureaucracyStatus, AdministrativeComplexity, process_bureaucracy};
use scale::layer1::bureaucracy::{AutonomousEdict, SpawnEdictEvent};

#[test]
fn test_high_complexity_spawns_autonomous_edict() {
    let mut app = App::new();
    app.insert_resource(BureaucracyStatus {
        complexity: AdministrativeComplexity(150.0), // Above threshold
        backlog: 50.0,
    });
    app.add_event::<SpawnEdictEvent>();
    app.add_systems(Update, process_bureaucracy);

    app.update();

    let events = app.world().resource::<Events<SpawnEdictEvent>>();
    let reader = events.get_cursor();
    assert!(reader.len(&events) > 0, "High complexity should trigger an edict");
}

#[test]
fn test_edict_causes_accidents_on_violation() {
    let mut app = App::new();
    // Simulate an active edict and a system that checks for violations
    // e.g., "All red items in North".
    // For test, we mock a violation checking system.
    use scale::layer1::bureaucracy::{ActiveEdicts, check_edict_violations, EdictViolationEvent};

    app.insert_resource(ActiveEdicts(vec![AutonomousEdict::MoveItemsNorth]));
    app.add_event::<EdictViolationEvent>();
    app.add_systems(Update, check_edict_violations);

    // Inject state to cause violation (e.g., resource not in north)
    // For dummy test, we'll assume the mock setup guarantees a violation.
    app.update();

    let events = app.world().resource::<Events<EdictViolationEvent>>();
    let reader = events.get_cursor();
    assert!(reader.len(&events) > 0, "Violating an active edict must trigger an accident/violation event");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/bureaucracy.rs
use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct BureaucracyStatus {
    pub complexity: AdministrativeComplexity,
    pub backlog: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdministrativeComplexity(pub f32);

#[derive(Event, Debug, Clone)]
pub struct SpawnEdictEvent;

#[derive(Resource, Debug, Clone)]
pub struct ActiveEdicts(pub Vec<AutonomousEdict>);

#[derive(Debug, Clone, PartialEq)]
pub enum AutonomousEdict {
    MoveItemsNorth,
    HaltProductionTuesdays,
}

#[derive(Event, Debug, Clone)]
pub struct EdictViolationEvent;

pub fn process_bureaucracy(
    status: Res<BureaucracyStatus>,
    mut edict_events: EventWriter<SpawnEdictEvent>,
) {
    if status.complexity.0 > 100.0 {
        edict_events.send(SpawnEdictEvent);
    }
}

pub fn check_edict_violations(
    edicts: Res<ActiveEdicts>,
    mut violation_events: EventWriter<EdictViolationEvent>,
) {
    for edict in &edicts.0 {
        if *edict == AutonomousEdict::MoveItemsNorth {
            // Fake checking logic: assume it always fails for green pass
            violation_events.send(EdictViolationEvent);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Complexity Source**: `AdministrativeComplexity` should be dynamically calculated from the number of concurrent policies, population size, and active faction edicts.
- **Accident Generation**: When an `EdictViolationEvent` fires, it should hook into the existing injury or building malfunction systems.
- **UI Exposure**: The active `AutonomousEdict` must be visible to the player so they understand why things are breaking.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Administrative complexity above a threshold triggers autonomous edicts.

## 7. Technical Guidance
- `BureaucracyStatus` might be better as a `Component` on the `Colony` entity if it exists, or keeping it as a `Resource` is fine.
- Add actual spatial/inventory checking for the `MoveItemsNorth` edict during implementation.

## 8. Questions
*Builder: add questions here if spec is unclear.*
