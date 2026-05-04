# 1251: The Empathy Amplifier

## 1. Overview
An endgame structure that broadcasts the colony's average Morale into Layer 2/3. Extreme Joy attracts massive immigration and cultural victory points. Extreme Despair creates a "Psychic Hazard" that damages enemy fleets.

## 2. Dependencies
- 005 Pop Needs (Morale calculation)
- 016 Utility AI System
- Layer 2/3 basic integration (for the effects of the broadcast)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Morale;
    use crate::layer1::architecture::Building;

    #[test]
    fn test_empathy_amplifier_extreme_joy() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, emit_psychic_broadcast_system);

        let pop = app.world_mut().spawn(Morale { value: 100.0 }).id();
        let amplifier = app.world_mut().spawn((
            Building { building_type: BuildingType::EmpathyAmplifier },
            EmpathyAmplifier { operational: true }
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<PsychicBroadcastEvent>>();
        let mut reader = events.get_reader();
        let emitted_events: Vec<_> = reader.read(events).collect();

        assert_eq!(emitted_events.len(), 1);
        assert_eq!(emitted_events[0].broadcast_type, BroadcastType::ExtremeJoy);
        assert_eq!(emitted_events[0].source_entity, amplifier);
    }

    #[test]
    fn test_empathy_amplifier_extreme_despair() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, emit_psychic_broadcast_system);

        let pop = app.world_mut().spawn(Morale { value: 0.0 }).id();
        let amplifier = app.world_mut().spawn((
            Building { building_type: BuildingType::EmpathyAmplifier },
            EmpathyAmplifier { operational: true }
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<PsychicBroadcastEvent>>();
        let mut reader = events.get_reader();
        let emitted_events: Vec<_> = reader.read(events).collect();

        assert_eq!(emitted_events.len(), 1);
        assert_eq!(emitted_events[0].broadcast_type, BroadcastType::PsychicHazard);
        assert_eq!(emitted_events[0].source_entity, amplifier);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN

#[derive(Component)]
pub struct EmpathyAmplifier {
    pub operational: bool,
}

#[derive(Event, Debug, PartialEq)]
pub struct PsychicBroadcastEvent {
    pub source_entity: Entity,
    pub broadcast_type: BroadcastType,
}

#[derive(Debug, PartialEq)]
pub enum BroadcastType {
    ExtremeJoy,
    PsychicHazard,
}

pub fn emit_psychic_broadcast_system(
    mut events: EventWriter<PsychicBroadcastEvent>,
    amplifiers: Query<(Entity, &EmpathyAmplifier)>,
    pops: Query<&crate::layer1::needs::Morale>,
) {
    let mut total_morale = 0.0;
    let mut pop_count = 0;

    for morale in pops.iter() {
        total_morale += morale.value;
        pop_count += 1;
    }

    if pop_count == 0 { return; }

    let avg_morale = total_morale / pop_count as f32;

    for (entity, amplifier) in amplifiers.iter() {
        if !amplifier.operational { continue; }

        if avg_morale >= 90.0 {
            events.send(PsychicBroadcastEvent {
                source_entity: entity,
                broadcast_type: BroadcastType::ExtremeJoy,
            });
        } else if avg_morale <= 10.0 {
            events.send(PsychicBroadcastEvent {
                source_entity: entity,
                broadcast_type: BroadcastType::PsychicHazard,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate colony average morale once globally per tick in a separate system, so the Empathy Amplifier just reads a `ColonyMorale` resource instead of iterating over all pops.
- Add cooldowns so the broadcast doesn't fire every single frame. We should use a `Timer` component on the `EmpathyAmplifier`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Integrate the event emission with Layer 2/3 so that fleets and immigration queues can consume the `PsychicBroadcastEvent`.
- Ensure `EmpathyAmplifier` consumes massive power. Use existing energy grid mechanics.

## 8. Questions
*Builder: add questions here if spec is unclear.*
