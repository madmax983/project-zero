1. **Explore & Define Modules**:
    - Execute the following:
      ```bash
      cat << 'INNER_EOF' > src/layer1/social/golden_age.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::pop::Speed;
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
            Speed { base: 10.0, current: 10.0, accumulator: 0.0 },
        )).id();

        app.update();

        let speed = app.world().get::<Speed>(pop_entity).unwrap();
        assert!(speed.current < speed.base, "High complacency should reduce current movement speed");
    }

    #[test]
    fn test_complacency_ignores_low_priority_alerts() {
        let mut app = App::new();
        app.add_event::<AlertEvent>();
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
INNER_EOF
      ```
    - Verify the file creation with `cat src/layer1/social/golden_age.rs`.

2. **Implement Systems** (GREEN Phase):
    - Execute the following:
      ```bash
      cat << 'INNER_EOF' > src/layer1/social/golden_age.rs
use bevy_ecs::prelude::*;
use crate::layer1::pop::Speed;

#[derive(Component)]
pub struct NeedsFulfilled {
    pub fully_satisfied: bool,
}

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

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AlertResponseState {
    Idle,
    Responding,
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
    mut query: Query<(&Complacency, &mut Speed)>
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::pop::Pop;
    use crate::layer1::pop::Speed;

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
            Speed { base: 10.0, current: 10.0, accumulator: 0.0 },
        )).id();

        app.update();

        let speed = app.world().get::<Speed>(pop_entity).unwrap();
        assert!(speed.current < speed.base, "High complacency should reduce current movement speed");
    }

    #[test]
    fn test_complacency_ignores_low_priority_alerts() {
        let mut app = App::new();
        app.add_event::<AlertEvent>();
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
INNER_EOF
      ```
    - Verify the file update with `cat src/layer1/social/golden_age.rs`.

3. **Integration & Refactor**:
    - Export `golden_age.rs` by running `echo 'pub mod golden_age;' >> src/layer1/social/mod.rs` and `echo 'pub use golden_age::*;' >> src/layer1/social/mod.rs`. Verify this change with `tail src/layer1/social/mod.rs`.
    - Register the new systems in `src/layer1/systems/observation.rs` by writing a python script `patch.py` that replaces `crate::layer1::social::cadet::death_consequence_system,` with `crate::layer1::social::cadet::death_consequence_system,\ncrate::layer1::social::golden_age::complacency_accumulation_system,\ncrate::layer1::social::golden_age::apply_complacency_debuffs_system,\ncrate::layer1::social::golden_age::pop_alert_response_system,`. Execute it and verify with `git diff`.
4. **Testing**:
    - Run `cargo test` to verify the module works as designed.
5. **Pre-commit**:
    - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
