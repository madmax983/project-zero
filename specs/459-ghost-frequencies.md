# Ghost Frequencies

## 1. Overview
Adding a new "Quantum Comm Array" mechanic that periodically receives broadcasts from parallel universe versions of the colony. These broadcasts might warn of incoming disasters or grant free research, but trusting them inherently risks misdirection if the parallel timeline has diverged too far. The goal is to provide a high-risk, high-reward tension point: trusting unverifiable, potentially life-saving information from a different reality versus ignoring it and risking annihilation.

## 2. Dependencies
- Base `Building` components.
- Events system for disasters (`DisasterWarningEvent`).
- Research system backend (`TechUnlockEvent`).
- Resource cost structures for array operation (Energy).

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_quantum_array_receives_broadcast() {
        // Arrange
        let mut app = App::new();
        app.add_event::<QuantumBroadcastEvent>();
        app.add_systems(Update, process_quantum_array_system);

        // Spawn Quantum Comm Array with power
        app.world_mut().spawn((
            QuantumCommArray { signal_strength: 1.0 },
            Powered,
        ));

        // Act
        // Advance time enough to trigger a broadcast
        app.update();

        // Assert
        let events = app.world().resource::<Events<QuantumBroadcastEvent>>();
        assert_eq!(events.len(), 1, "Array should receive a broadcast when powered.");
    }

    #[test]
    fn test_unpowered_array_no_broadcast() {
        // Arrange
        let mut app = App::new();
        app.add_event::<QuantumBroadcastEvent>();
        app.add_systems(Update, process_quantum_array_system);

        // Spawn Quantum Comm Array WITHOUT power
        app.world_mut().spawn(QuantumCommArray { signal_strength: 1.0 });

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<QuantumBroadcastEvent>>();
        assert_eq!(events.len(), 0, "Unpowered array should not receive broadcasts.");
    }

    #[test]
    fn test_broadcast_triggers_false_alarm_disaster() {
         // Arrange
        let mut app = App::new();
        app.add_event::<DisasterWarningEvent>();
        app.add_systems(Update, handle_quantum_broadcast_system);

        // Inject a parallel broadcast that warns of disaster but timeline diverged
        app.world_mut().send_event(QuantumBroadcastEvent {
            broadcast_type: BroadcastType::DisasterWarning,
            divergence_level: 0.8, // High divergence = false alarm likely
        });

        // Act
        app.update();

        // Assert
        let warnings = app.world().resource::<Events<DisasterWarningEvent>>();
        assert_eq!(warnings.len(), 1, "Warning should be generated.");

        // The actual disaster system (tested elsewhere) would use divergence_level to cancel the disaster,
        // resulting in the player wasting resources on prep.
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct QuantumCommArray {
    pub signal_strength: f32,
}

#[derive(Component)]
pub struct Powered;

#[derive(Event)]
pub struct QuantumBroadcastEvent {
    pub broadcast_type: BroadcastType,
    pub divergence_level: f32,
}

#[derive(Clone, Copy)]
pub enum BroadcastType {
    DisasterWarning,
    TechBlueprint,
}

#[derive(Event)]
pub struct DisasterWarningEvent {
    pub divergence_level: f32,
}

pub fn process_quantum_array_system(
    query: Query<&QuantumCommArray, With<Powered>>,
    mut events: EventWriter<QuantumBroadcastEvent>,
) {
    for _array in query.iter() {
        // In minimal implementation, trigger every tick for test to pass
        events.send(QuantumBroadcastEvent {
            broadcast_type: BroadcastType::DisasterWarning,
            divergence_level: 0.5,
        });
    }
}

pub fn handle_quantum_broadcast_system(
    mut broadcast_events: EventReader<QuantumBroadcastEvent>,
    mut warning_events: EventWriter<DisasterWarningEvent>,
) {
    for event in broadcast_events.read() {
        if let BroadcastType::DisasterWarning = event.broadcast_type {
            warning_events.send(DisasterWarningEvent {
                divergence_level: event.divergence_level,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Timers:** Use a `Timer` inside `QuantumCommArray` so it doesn't broadcast every single tick.
- **RNG:** Inject random divergence levels and broadcast types using `rand` or `bevy_turborand`.
- **System ordering:** Ensure `process_quantum_array_system` runs in the appropriate system set (e.g., `Layer1SystemSet::Observation`).
- **Data Preservation:** We need to persist the divergence level so the disaster system knows whether to actually spawn the pirate raid/meteor or not.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new code.
- [ ] A `QuantumCommArray` must be `Powered` to receive a `QuantumBroadcastEvent`.
- [ ] High divergence broadcasts correctly fail to materialize their promised outcomes (disasters don't happen, or tech doesn't perfectly apply).

## 7. Technical Guidance
- Add `QuantumCommArray` to the building registry in the layer 1 economy module.
- Create a clear visual/UI indicator for the player when a broadcast is received, explicitly showing a "Divergence Estimate" so they have a hint of the risk.
- Integrate with existing disaster queues. If the array warns of a raid, enqueue it, but attach the divergence flag. If divergence > threshold, silently cancel the raid when the timer pops.

## 8. Questions
*Builder: add questions here if spec is unclear.*
