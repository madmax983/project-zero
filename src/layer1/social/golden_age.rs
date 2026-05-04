use crate::layer1::pop::Speed;
use bevy_ecs::prelude::*;

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
    pub is_high_priority: bool,
    pub message: String,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RespondingToAlert;

pub fn complacency_accumulation_system(
    safety: Res<ColonySafety>,
    mut query: Query<(&mut Complacency, &NeedsFulfilled)>,
) {
    if safety.days_without_incident > 30 {
        for (mut complacency, needs) in query.iter_mut() {
            if needs.fully_satisfied {
                complacency.level = (complacency.level + 0.5).min(100.0);
            }
        }
    }
}

pub fn apply_complacency_debuffs_system(mut query: Query<(&Complacency, &mut Speed)>) {
    for (complacency, mut speed) in query.iter_mut() {
        // At 100 complacency, speed is reduced by 20%
        let reduction = 1.0 - (complacency.level / 100.0) * 0.2;
        speed.current = speed.base * reduction;
    }
}

pub fn pop_alert_response_system(
    mut commands: Commands,
    mut events: EventReader<AlertEvent>,
    query: Query<(Entity, &Complacency)>,
) {
    for event in events.read() {
        for (entity, complacency) in query.iter() {
            if !event.is_high_priority && complacency.level > 50.0 {
                // Ignore the alert
                continue;
            }

            // Otherwise, respond to alert
            commands.entity(entity).insert(RespondingToAlert);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_complacency_increases_during_peace() {
        let mut app = App::new();
        app.insert_resource(ColonySafety {
            days_without_incident: 100,
        });
        app.add_systems(Update, complacency_accumulation_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Complacency { level: 0.0 },
                NeedsFulfilled {
                    fully_satisfied: true,
                },
            ))
            .id();

        app.update();

        let complacency = app.world().get::<Complacency>(pop_entity).unwrap();
        assert!(
            complacency.level > 0.0,
            "Complacency should increase when needs are met and colony is safe"
        );
    }

    #[test]
    fn test_complacency_reduces_movement_speed() {
        let mut app = App::new();
        app.add_systems(Update, apply_complacency_debuffs_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Complacency { level: 100.0 }, // Max complacency
                Speed {
                    base: 10.0,
                    current: 10.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        app.update();

        let speed = app.world().get::<Speed>(pop_entity).unwrap();
        assert!(
            speed.current < speed.base,
            "High complacency should reduce current movement speed"
        );
    }

    #[test]
    fn test_complacency_ignores_low_priority_alerts() {
        let mut app = App::new();
        app.add_event::<AlertEvent>();
        app.add_systems(Update, pop_alert_response_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Complacency { level: 80.0 }, // Highly complacent
            ))
            .id();

        // Trigger a low priority alert
        app.world_mut().send_event(AlertEvent {
            is_high_priority: false,
            message: "Minor leak detected".to_string(),
        });

        app.update();

        assert!(
            app.world().get::<RespondingToAlert>(pop_entity).is_none(),
            "Complacent pop should ignore low priority alert"
        );
    }
}
