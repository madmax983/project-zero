# Spec 609: The Golden Age

## 1. Overview
**Layer:** 1
**Fantasy:** The danger of peace. Hard times create strong men; good times create soft men.
**Mechanic:** Long periods of high safety and fulfilled needs generate "Complacency". Complacent pops have high mood but reduced movement speed, slower skill gain, and ignore "Low Priority" alerts.
**Emergence:** A raid occurs during the "Golden Age". The siren wails, but the pops slowly finish their meals before moving to the bunkers. They are slaughtered because they forgot fear.
**Tension:** Maintain a state of low-level crisis to keep the edge, or allow paradise and risk vulnerability?

## 2. Dependencies
- `src/layer1/pop.rs` (Pop component)
- `src/layer1/needs.rs` (Checking if all needs are fulfilled)
- `src/layer1/morale.rs` (High mood generation)
- `src/layer1/movement.rs` and `src/layer1/skills.rs` (Applying movement speed and skill gain penalties)
- `src/layer1/alert.rs` or `src/layer1/security/` (Ignoring low priority alarms)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod golden_age_tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_complacency_accumulates_during_high_safety() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_complacency_system);
        app.insert_resource(ColonySafety { level: SafetyLevel::High, duration_ticks: 1000 });

        let pop_entity = app.world_mut().spawn((
            Pop,
            Complacency { level: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let complacency = app.world().get::<Complacency>(pop_entity).unwrap();
        assert!(complacency.level > 0.0, "Complacency should increase when colony safety is consistently high.");
    }

    #[test]
    fn test_complacency_drops_during_crisis() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_complacency_system);
        app.insert_resource(ColonySafety { level: SafetyLevel::Danger, duration_ticks: 10 });

        let pop_entity = app.world_mut().spawn((
            Pop,
            Complacency { level: 50.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let complacency = app.world().get::<Complacency>(pop_entity).unwrap();
        assert!(complacency.level < 50.0, "Complacency should rapidly decrease during a crisis.");
    }

    #[test]
    fn test_high_complacency_applies_movement_penalty() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_complacency_penalties_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Complacency { level: 80.0 }, // High enough to trigger penalty
            MovementSpeed { base: 10.0, current: 10.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let speed = app.world().get::<MovementSpeed>(pop_entity).unwrap();
        assert!(speed.current < speed.base, "Movement speed should be penalized when complacent.");
    }

    #[test]
    fn test_high_complacency_ignores_low_priority_alerts() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_alerts_system);
        app.add_event::<AlertEvent>();

        let pop_entity = app.world_mut().spawn((
            Pop,
            Complacency { level: 90.0 },
            AlertResponse { is_responding: false },
        )).id();

        app.world_mut().resource_mut::<Events<AlertEvent>>().send(AlertEvent {
            priority: AlertPriority::Low,
        });

        // Act
        app.update();

        // Assert
        let response = app.world().get::<AlertResponse>(pop_entity).unwrap();
        assert!(!response.is_responding, "Complacent pops should ignore low priority alerts.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Complacency {
    pub level: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct MovementSpeed {
    pub base: f32,
    pub current: f32,
}

#[derive(Component)]
pub struct AlertResponse {
    pub is_responding: bool,
}

#[derive(PartialEq)]
pub enum SafetyLevel {
    High,
    Danger,
}

#[derive(Resource)]
pub struct ColonySafety {
    pub level: SafetyLevel,
    pub duration_ticks: u32,
}

pub enum AlertPriority {
    Low,
    High,
}

#[derive(Event)]
pub struct AlertEvent {
    pub priority: AlertPriority,
}

pub fn process_complacency_system(
    safety: Res<ColonySafety>,
    mut query: Query<&mut Complacency, With<Pop>>,
) {
    let increase_rate = 0.5;
    let decrease_rate = 5.0;

    for mut complacency in query.iter_mut() {
        if safety.level == SafetyLevel::High && safety.duration_ticks > 500 {
            complacency.level = (complacency.level + increase_rate).min(100.0);
        } else if safety.level == SafetyLevel::Danger {
            complacency.level = (complacency.level - decrease_rate).max(0.0);
        }
    }
}

pub fn apply_complacency_penalties_system(
    mut query: Query<(&Complacency, &mut MovementSpeed), With<Pop>>,
) {
    for (complacency, mut speed) in query.iter_mut() {
        if complacency.level > 75.0 {
            // Apply a 20% speed penalty
            speed.current = speed.base * 0.8;
        } else {
            speed.current = speed.base;
        }
    }
}

pub fn process_alerts_system(
    mut alert_events: EventReader<AlertEvent>,
    mut query: Query<(&Complacency, &mut AlertResponse), With<Pop>>,
) {
    for event in alert_events.read() {
        for (complacency, mut response) in query.iter_mut() {
            if event.priority == AlertPriority::Low && complacency.level > 75.0 {
                // Ignore the alert
                response.is_responding = false;
            } else {
                response.is_responding = true;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Hook `ColonySafety` resource updates directly into the layer 1 combat and disaster event streams (e.g., `RaidStartedEvent`, `FireOutbreakEvent` should immediately reset safety to `Danger`).
- **Refactor Opportunity:** The `MovementSpeed` penalty should ideally be integrated into a unified stat modifier system (`StatBuff`/`StatDebuff` components) rather than manually overwriting `current` every tick, which risks race conditions with other speed-altering effects (like injuries or drugs).
- **Behavioral Expansion:** Complacent pops could have specific new idle animations (e.g., lounging, walking slowly) or chat logs reflecting their lack of concern.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Long periods without danger increase the `Complacency` stat on Pops.
- [ ] High `Complacency` effectively ignores `Low` priority alerts and reduces movement speed.

## 7. Technical Guidance
- Implement this logic within a new module `src/layer1/complacency.rs` or integrate it deeply into `src/layer1/morale.rs`.
- Ensure the `AlertResponse` logic accurately hooks into the Utility AI (`src/layer1/utility_ai.rs`), preventing the AI from prioritizing the bunker/rally point action when ignoring the alert.
- The `ColonySafety` resource needs a global ticking system to track how many days/ticks the colony has gone without a major negative event.

## 8. Questions
*Builder: add questions here if spec is unclear.*
