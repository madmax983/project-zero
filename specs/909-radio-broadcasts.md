# 909: Radio Broadcasts

## 1. Overview
Colonies can build Comms Arrays to broadcast signals into the void (e.g., Trade, Distress, Intimidation). These signals attract specific ship types to the system, but also raise the "System Threat" level, drawing unwanted attention from hostiles like Pirates or Reavers.

## 2. Dependencies
- `layer1::infrastructure::CommsArray`
- `layer2::system_map::SystemThreat`
- `layer3::entities::ShipFleet`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::system_map::SystemThreat;

    #[test]
    fn test_broadcast_signal_increases_threat() {
        let mut app = App::new();
        app.add_systems(Update, process_comms_broadcasts);

        let system = app.world_mut().spawn(SystemThreat { level: 0.0 }).id();

        let comms = app.world_mut().spawn((
            CommsArray { active_signal: Some(SignalType::Distress) },
            LocatedIn(system),
        )).id();

        // Act
        app.update();

        // Assert
        let threat = app.world().get::<SystemThreat>(system).unwrap();
        assert!(threat.level > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Simplest implementation to turn tests green
```

## 5. REFACTOR Phase: Quality & Design
- Create discrete events for `SignalBroadcastEvent` rather than continually checking `active_signal` on every tick.
- Group threat calculations based on signal type.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Active comms arrays increase system threat

## 7. Technical Guidance
- Ensure that the generated threat affects Layer 3 spawn probabilities (e.g. higher threat = higher chance of Pirate fleet spawning targeting the system).
- Different signals (Distress vs Intimidation) might have varying threat multipliers.

## 8. Questions
*Builder: add questions here if spec is unclear.*
