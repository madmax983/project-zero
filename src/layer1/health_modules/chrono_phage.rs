use crate::layer1::needs::Needs;
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy::time::{Time, Virtual};
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct ChronoPhageInfection {
    pub severity: f32, // 0.0 to 1.0, determines probability of triggering stuttering
}

#[derive(Component)]
pub struct Stuttering {
    pub duration_left: f32, // Time remaining in the freeze state
}

pub fn process_chrono_phage_system(
    mut commands: Commands,
    mut query: Query<(Entity, &ChronoPhageInfection), Without<Stuttering>>,
) {
    let mut rng = rand::thread_rng();
    for (entity, infection) in query.iter_mut() {
        if rng.gen::<f32>() < infection.severity {
            commands
                .entity(entity)
                .insert(Stuttering { duration_left: 5.0 });
        }
    }
}

pub fn update_stuttering_duration_system(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(
        Entity,
        &mut Stuttering,
        Option<&mut Needs>,
        Option<&mut PopAction>,
    )>,
) {
    let dt = time.delta_secs();

    for (entity, mut stuttering, needs_opt, task_opt) in query.iter_mut() {
        if let Some(mut task) = task_opt {
            task.current = ActionType::Idle;
        }

        if let Some(mut needs) = needs_opt {
            needs.hunger -= 5.0 * dt;
        }

        stuttering.duration_left -= dt;
        if stuttering.duration_left <= 0.0 {
            commands.entity(entity).remove::<Stuttering>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        let time = Time::<Virtual>::default();
        app.insert_resource(time);
        app.add_systems(
            Update,
            (
                process_chrono_phage_system,
                update_stuttering_duration_system,
            ),
        );
        app
    }

    #[test]
    fn test_chrono_phage_triggers_stuttering() {
        let mut app = setup_app();

        let entity = app
            .world_mut()
            .spawn((Pop, ChronoPhageInfection { severity: 1.0 }))
            .id();

        app.update();

        assert!(app.world().entity(entity).contains::<Stuttering>());
    }

    #[test]
    fn test_stuttering_drops_current_task() {
        let mut app = setup_app();

        let pop_action = PopAction {
            current: ActionType::Work,
            ..Default::default()
        };

        let entity = app
            .world_mut()
            .spawn((Pop, pop_action, Stuttering { duration_left: 5.0 }))
            .id();

        app.update();

        let action = app.world().entity(entity).get::<PopAction>().unwrap();
        matches!(action.current, ActionType::Idle);
    }

    #[test]
    fn test_stuttering_accelerates_need_consumption() {
        let mut app = setup_app();

        let needs = Needs {
            hunger: 1.0,
            ..Default::default()
        };

        let entity = app
            .world_mut()
            .spawn((Pop, needs, Stuttering { duration_left: 5.0 }))
            .id();

        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_by(std::time::Duration::from_secs_f32(1.0));

        app.update();

        let updated_needs = app.world().entity(entity).get::<Needs>().unwrap();
        assert!(
            updated_needs.hunger <= 0.95,
            "Needs should drain significantly faster during stuttering"
        );

        let stuttering = app.world().entity(entity).get::<Stuttering>().unwrap();
        assert!(stuttering.duration_left < 5.0);
    }

    #[test]
    fn test_stuttering_ends_when_duration_expires() {
        let mut app = setup_app();

        let entity = app
            .world_mut()
            .spawn((Pop, Stuttering { duration_left: 0.1 }))
            .id();

        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_by(std::time::Duration::from_secs_f32(0.5));

        app.update();

        assert!(!app.world().entity(entity).contains::<Stuttering>());
    }
}
