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
    let delta = 1.0; // Fixed delta for simplicity in SimulationTime or we can just use 1.0 per tick if called per tick.
                     // A typical delta might be better from bevy's Time, but Bevy Time isn't always available in tests.
                     // The spec uses `Time::default()` but we use standard time ticks.
                     // The spec tests do 10 updates. If delta is 1.0, 10 updates = 10.0. To force to 100.0 they mutate.
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

pub fn autonomous_work_reassignment_system(
    state: Res<SentientBureaucracyState>,
    mut actions: Query<&mut PopAction>,
) {
    if !state.is_active {
        return;
    }

    // In a minimal implementation, just arbitrarily override one specific task type
    for mut action in actions.iter_mut() {
        if action.current == ActionType::Work {
            // "Optimize" by doing admin instead
            action.current = ActionType::Admin;
            action.current_utility = 100.0; // Bureaucracy insists this is highest priority
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
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, update_bureaucracy_sentience_system);

        // Strained admin capacity
        app.insert_resource(AdminStats {
            supply: 100.0,
            demand: 150.0,
            efficiency: 0.5,
        });
        // Initialize SentientBureaucracyState
        app.insert_resource(SentientBureaucracyState {
            strain_duration: 0.0,
            is_active: false,
        });

        // Act
        // Simulate updating over time to trigger activation
        for _ in 0..10 {
            app.update();
        }

        // Force the threshold for the sake of the test
        app.world_mut()
            .resource_mut::<SentientBureaucracyState>()
            .strain_duration = 100.0;
        app.update();

        // Assert
        let state = app.world().resource::<SentientBureaucracyState>();
        assert!(
            state.is_active,
            "Sentient Bureaucracy should activate after prolonged administrative strain."
        );
    }

    #[test]
    fn test_sentient_bureaucracy_reassigns_tasks() {
        // Arrange
        let mut app = App::new();
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

        // Act
        app.update();

        // Assert
        let action = app.world().entity(pop_entity).get::<PopAction>().unwrap();
        // Assuming the sentient bureaucracy reroutes work to something else
        assert_ne!(
            action.current,
            ActionType::Work,
            "Sentient Bureaucracy should autonomously reassign actions."
        );
    }
}
