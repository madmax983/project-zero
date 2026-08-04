use crate::layer1::administration::admin::AdminStats;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct SentientBureaucracyState {
    pub strain_duration: f32,
    pub is_active: bool,
}

const STRAIN_THRESHOLD: f32 = 50.0;

pub fn update_bureaucracy_sentience_system(
    admin_stats: Option<Res<AdminStats>>,
    mut state: ResMut<SentientBureaucracyState>,
    // In our tests, time might not exist, but let's use a standard 1.0 if not there or rely on SimulationTime
    _time: Option<Res<SimulationTime>>,
) {
    let delta = 1.0;
    let efficiency = admin_stats.map(|s| s.efficiency).unwrap_or(1.0);

    if efficiency < 1.0 {
        state.strain_duration += delta;
        if state.strain_duration >= STRAIN_THRESHOLD {
            state.is_active = true;
        }
    } else {
        // Slowly recover if under capacity
        state.strain_duration -= delta * 2.0;
        if state.strain_duration < 0.0 {
            state.strain_duration = 0.0;
            state.is_active = false;
        }
    }
}

#[derive(Event)]
pub struct TaskAdministrativelyOptimizedEvent {
    pub entity: Entity,
}

pub fn autonomous_work_reassignment_system(
    state: Res<SentientBureaucracyState>,
    mut actions: Query<(Entity, &mut PopAction)>,
    mut event_writer: EventWriter<TaskAdministrativelyOptimizedEvent>,
) {
    if !state.is_active {
        return;
    }

    for (entity, mut action) in actions.iter_mut() {
        if action.current == ActionType::Work {
            action.current = ActionType::Admin;
            action.current_utility = 100.0;
            event_writer.send(TaskAdministrativelyOptimizedEvent { entity });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::mind::utility_types::{ActionType, PopAction};
    use bevy_app::{App, Update};

    #[test]
    fn test_sentient_bureaucracy_activation_after_prolonged_strain() {
        let mut app = App::new();
        app.add_systems(Update, update_bureaucracy_sentience_system);

        app.insert_resource(AdminStats {
            supply: 100.0,
            demand: 150.0,
            efficiency: 0.5,
        });
        app.insert_resource(SentientBureaucracyState {
            strain_duration: 0.0,
            is_active: false,
        });

        for _ in 0..10 {
            app.update();
        }

        app.world_mut()
            .resource_mut::<SentientBureaucracyState>()
            .strain_duration = 100.0;
        app.update();

        let state = app.world().resource::<SentientBureaucracyState>();
        assert!(
            state.is_active,
            "Sentient Bureaucracy should activate after prolonged administrative strain."
        );
    }

    #[test]
    fn test_sentient_bureaucracy_reassigns_tasks() {
        let mut app = App::new();
        app.add_event::<TaskAdministrativelyOptimizedEvent>();
        app.add_systems(Update, autonomous_work_reassignment_system);

        app.insert_resource(SentientBureaucracyState {
            strain_duration: 100.0,
            is_active: true,
        });

        let pop_entity = app
            .world_mut()
            .spawn(PopAction {
                current: ActionType::Work,
                current_utility: 10.0,
                ticks_committed: 0,
            })
            .id();

        app.update();

        let action = app.world().entity(pop_entity).get::<PopAction>().unwrap();
        assert_ne!(
            action.current,
            ActionType::Work,
            "Sentient Bureaucracy should autonomously reassign actions."
        );

        let events = app
            .world()
            .resource::<Events<TaskAdministrativelyOptimizedEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).count(),
            1,
            "Should emit optimization event"
        );
    }
}
