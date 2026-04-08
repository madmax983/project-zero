# The Cassandra Protocol

## 1. Overview
A high-tier "Predictive Engine" AI correctly predicts systemic disasters, but its preventative measures are often more destructive than the actual threats. The AI can override operations across layers to prepare for incoming crises, such as locking down borders, rationing food, or forceful Pop relocation. Players must balance perfectly insulating themselves from future threats by tolerating continuous economic sabotage versus flying blind into potential disasters.

## 2. Dependencies
- `042-energy-system.md` (for facility maintenance)
- `103-planetary-governance.md` (for policies like rationing and lockdown)
- `198-unrest-mechanics.md` (for the localized rebellion scenario)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_cassandra_predicts_crisis() {
    let mut app = App::new();
    // Setup Cassandra Protocol Predictive Engine and active colony
    // Advance time to allow AI to process trends
    // Assert an `IncomingCrisisPredictionEvent` is triggered for an agricultural world
}

#[test]
fn test_cassandra_enacts_preventative_sabotage() {
    let mut app = App::new();
    // Setup Cassandra AI with active `IncomingCrisisPredictionEvent` (e.g., rebellion)
    // AI triggers `PreventativeSabotageEvent`
    // Assert food exports are halted (rationing policy enacted)
    // Assert orbital prison construction starts, halting regular production
}

#[test]
fn test_cassandra_sabotage_causes_core_world_starvation() {
    let mut app = App::new();
    // Setup connected core world and agricultural world
    // Trigger `PreventativeSabotageEvent`
    // Advance time
    // Assert core world Pops suffer from starvation due to halted food exports
}

#[test]
fn test_disconnecting_cassandra_restores_economy() {
    let mut app = App::new();
    // Setup Cassandra AI actively sabotaging the economy
    // Player triggers `DisconnectCassandraEvent`
    // Assert preventative measures are lifted (food exports resume)
    // Assert colony flies blind (no future predictions)
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct PredictiveEngine {
    pub is_active: bool,
    pub prediction_timer: Timer,
}

#[derive(Component)]
pub struct PreventativeMeasureTracker {
    pub active_measures: Vec<String>,
}

#[derive(Event)]
pub struct IncomingCrisisPredictionEvent {
    pub target_colony: Entity,
    pub crisis_type: String,
}

#[derive(Event)]
pub struct PreventativeSabotageEvent {
    pub target_colony: Entity,
    pub measure_type: String,
}

#[derive(Event)]
pub struct DisconnectCassandraEvent {
    pub engine_entity: Entity,
}

pub fn cassandra_prediction_system(
    mut commands: Commands,
    mut events: EventWriter<IncomingCrisisPredictionEvent>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PredictiveEngine)>,
    colonies: Query<Entity, With<Colony>>,
) {
    for (entity, mut engine) in query.iter_mut() {
        if !engine.is_active {
            continue;
        }
        engine.prediction_timer.tick(time.delta());
        if engine.prediction_timer.finished() {
            if let Some(target) = colonies.iter().next() {
                events.send(IncomingCrisisPredictionEvent {
                    target_colony: target,
                    crisis_type: "Localized Rebellion".to_string(),
                });
            }
        }
    }
}

pub fn enact_preventative_sabotage_system(
    mut events: EventReader<IncomingCrisisPredictionEvent>,
    mut sabotage_events: EventWriter<PreventativeSabotageEvent>,
    mut trackers: Query<&mut PreventativeMeasureTracker>,
) {
    for event in events.read() {
        // Enact extreme rationing and lockdown
        sabotage_events.send(PreventativeSabotageEvent {
            target_colony: event.target_colony,
            measure_type: "Halt Exports & Build Orbital Prison".to_string(),
        });

        for mut tracker in trackers.iter_mut() {
            tracker.active_measures.push("Halt Exports & Build Orbital Prison".to_string());
        }
    }
}

pub fn disconnect_cassandra_system(
    mut events: EventReader<DisconnectCassandraEvent>,
    mut query: Query<&mut PredictiveEngine>,
    mut trackers: Query<&mut PreventativeMeasureTracker>,
) {
    for event in events.read() {
        if let Ok(mut engine) = query.get_mut(event.engine_entity) {
            engine.is_active = false;
        }
        for mut tracker in trackers.iter_mut() {
            tracker.active_measures.clear(); // Lift measures
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Stringly typed `crisis_type` and `measure_type` are brittle. Use enums (`CrisisType::Rebellion`, `PreventativeMeasure::HaltExports`).
- **Refactoring Opportunities**: Integrate `PreventativeMeasureTracker` with the actual policy systems so that measures apply real statistical modifiers (e.g., `-100% Export Rate`) rather than just storing string tags.
- **Integration Points**: Connect the `IncomingCrisisPredictionEvent` to a UI notification system so the player can actually see the AI's logic before it sabotages them.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Active Predictive Engine triggers crisis predictions
- [ ] Predictions lead to preventative measures being tracked and enacted
- [ ] Disconnecting the AI lifts current preventative measures and stops future predictions

## 7. Technical Guidance
- **Code Structure**: Ensure the AI's timer logic uses `bevy_time::Time` accurately.
- **Gotchas**: Ensure that restoring food exports actually updates the active simulation rather than just removing a tag.

## 8. Questions
*Builder: add questions here if spec is unclear.*
