# Spec 633: The Golden Age

## 1. Overview
The danger of peace. Long periods of high safety and fulfilled needs generate "Complacency". Complacent pops have high mood but reduced movement speed, slower skill gain, and ignore "Low Priority" alerts. Maintaining a state of low-level crisis might keep the edge, whereas allowing a paradise risks extreme vulnerability during sudden raids.

## 2. Dependencies
- `src/layer1/pop.rs` (Pop components like Mood, Skills, Movement)
- `src/layer1/needs.rs` (Checking if needs are fulfilled)
- `src/layer1/alerts.rs` (Alert system for "Low Priority" vs "High Priority")
- `src/layer1/colony.rs` (Colony-wide safety tracking)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Mood, MovementSpeed};
    use crate::layer1::needs::NeedsFulfilled;

    #[test]
    fn test_complacency_increases_during_peace() {
        let mut app = App::new();
        app.insert_resource(ColonySafety { days_without_incident: 100 });
        app.add_systems(Update, complacency_accumulation_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Complacency { level: 0.0 },
            NeedsFulfilled { fully_satisfied: true },
        )).id();

        app.update();

        let complacency = app.world().get::<Complacency>(pop_entity).unwrap();
        assert!(complacency.level > 0.0, "Complacency should increase when needs are met and colony is safe");
    }

    #[test]
    fn test_complacency_reduces_movement_speed() {
        let mut app = App::new();
        app.add_systems(Update, apply_complacency_debuffs_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Complacency { level: 100.0 }, // Max complacency
            MovementSpeed { base: 10.0, current: 10.0 },
        )).id();

        app.update();

        let speed = app.world().get::<MovementSpeed>(pop_entity).unwrap();
        assert!(speed.current < speed.base, "High complacency should reduce current movement speed");
    }

    #[test]
    fn test_complacency_ignores_low_priority_alerts() {
        let mut app = App::new();
        app.add_systems(Update, pop_alert_response_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Complacency { level: 80.0 }, // Highly complacent
            AlertResponseState::Idle,
        )).id();

        // Trigger a low priority alert
        app.world_mut().send_event(AlertEvent {
            priority: AlertPriority::Low,
            message: "Minor leak detected".to_string(),
        });

        app.update();

        let state = app.world().get::<AlertResponseState>(pop_entity).unwrap();
        assert_eq!(*state, AlertResponseState::Idle, "Complacent pop should ignore low priority alert");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{MovementSpeed, AlertResponseState};
use crate::layer1::needs::NeedsFulfilled;

#[derive(Resource)]
pub struct ColonySafety {
    pub days_without_incident: u32,
}

#[derive(Component)]
pub struct Complacency {
    pub level: f32, // 0.0 to 100.0
}

#[derive(Event)]
pub struct AlertEvent {
    pub priority: AlertPriority,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AlertPriority {
    Low,
    High,
}

pub fn complacency_accumulation_system(
    safety: Res<ColonySafety>,
    mut query: Query<(&mut Complacency, &NeedsFulfilled)>
) {
    if safety.days_without_incident > 30 {
        for (mut complacency, needs) in query.iter_mut() {
            if needs.fully_satisfied {
                complacency.level = (complacency.level + 0.5).min(100.0);
            }
        }
    }
}

pub fn apply_complacency_debuffs_system(
    mut query: Query<(&Complacency, &mut MovementSpeed)>
) {
    for (complacency, mut speed) in query.iter_mut() {
        // At 100 complacency, speed is reduced by 20%
        let reduction = 1.0 - (complacency.level / 100.0) * 0.2;
        speed.current = speed.base * reduction;
    }
}

pub fn pop_alert_response_system(
    mut events: EventReader<AlertEvent>,
    mut query: Query<(&Complacency, &mut AlertResponseState)>
) {
    for event in events.read() {
        for (complacency, mut state) in query.iter_mut() {
            if event.priority == AlertPriority::Low && complacency.level > 50.0 {
                // Ignore the alert
                continue;
            }

            // Otherwise, respond to alert
            *state = AlertResponseState::Responding;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `Complacency` should ideally decay when a crisis happens (e.g., when an incident resets `days_without_incident` to 0).
- Integrate the skill gain reduction by checking `Complacency::level` in the skill progression system.
- `ColonySafety` could be an event-driven resource rather than ticking a counter linearly.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Complacency increases during long periods of peace with fulfilled needs.
- [ ] Complacency demonstrably reduces movement speed.
- [ ] Complacent pops ignore low-priority alerts.

## 7. Technical Guidance
- Ensure `days_without_incident` is reset correctly by raid, disaster, or famine events across the codebase.
- Review interaction with standard morale/mood buffs to ensure "Complacency" doesn't completely override the benefits of a well-run colony. It should be a trade-off.

## 8. Questions
*Builder: add questions here if spec is unclear.*
