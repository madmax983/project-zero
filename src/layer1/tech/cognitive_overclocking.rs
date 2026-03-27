use bevy_ecs::prelude::*;
use bevy::prelude::Time; // time needs to be imported here too since `process_neural_burnout_system` uses Time



pub const OVERCLOCK_SPEED_MULTIPLIER: f32 = 5.0;
pub const BURNOUT_THRESHOLD: f32 = 100.0;


#[derive(Component)]
pub struct WorkStats {
    pub base_speed: f32,
    pub current_speed: f32,
}

#[derive(Component)]
pub struct CognitiveOverclock {
    pub active: bool,
    pub time_active: f32,
}

#[derive(Component)]
pub struct NeuralTrauma {
    pub burnout_level: f32,
}

#[derive(Component, PartialEq, Debug)]
pub enum ActiveState {
    Working,
    Catatonic,
}

pub fn apply_cognitive_overclocking_system(
    mut query: Query<(&mut WorkStats, &CognitiveOverclock)>
) {
    for (mut stats, overclock) in query.iter_mut() {
        if overclock.active {
            stats.current_speed = stats.base_speed * OVERCLOCK_SPEED_MULTIPLIER;
        } else {
            stats.current_speed = stats.base_speed;
        }
    }
}

pub fn process_neural_burnout_system(
    time: Res<Time>,
    mut query: Query<(&mut NeuralTrauma, &mut CognitiveOverclock)>
) {
    for (mut trauma, mut overclock) in query.iter_mut() {
        if overclock.active {
            overclock.time_active += time.delta_secs();
            trauma.burnout_level += 1.0 * time.delta_secs(); // 1 burnout per second
        }
    }
}

pub fn check_burnout_threshold_system(
    mut query: Query<(&NeuralTrauma, &mut ActiveState)>
) {
    for (trauma, mut state) in query.iter_mut() {
        if trauma.burnout_level >= BURNOUT_THRESHOLD {
            *state = ActiveState::Catatonic;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::Pop;

    #[test]
    fn test_cognitive_overclocking_increases_work_speed() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_cognitive_overclocking_system);

        let entity = app.world_mut().spawn((
            Pop,
            WorkStats { base_speed: 1.0, current_speed: 1.0 },
            CognitiveOverclock { active: true, time_active: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let work_stats = app.world_mut().get::<WorkStats>(entity).unwrap();
        assert_eq!(work_stats.current_speed, 5.0); // 500% speed
    }

    #[test]
    fn test_cognitive_overclocking_accumulates_burnout() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Time::new_with(())); // Mock time
        app.add_systems(Update, process_neural_burnout_system);

        let entity = app.world_mut().spawn((
            Pop,
            CognitiveOverclock { active: true, time_active: 0.0 },
            NeuralTrauma { burnout_level: 0.0 },
        )).id();

        // Simulate some time passing
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(10));

        // Act
        app.update();

        // Assert
        let trauma = app.world_mut().get::<NeuralTrauma>(entity).unwrap();
        assert!(trauma.burnout_level > 0.0);
    }

    #[test]
    fn test_burnout_leads_to_catatonia() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, check_burnout_threshold_system);

        let entity = app.world_mut().spawn((
            Pop,
            NeuralTrauma { burnout_level: 100.0 }, // Past threshold
            ActiveState::Working,
        )).id();

        // Act
        app.update();

        // Assert
        let state = app.world_mut().get::<ActiveState>(entity).unwrap();
        assert_eq!(*state, ActiveState::Catatonic);
    }
}
