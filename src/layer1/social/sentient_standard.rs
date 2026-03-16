use crate::layer1::pop::Pop;
use crate::layer1::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Resource, Debug, Clone, Copy)]
pub struct SentientStandard {
    pub expected_luxury: f32,
    pub active: bool,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ColonyMetrics {
    pub current_luxury: f32,
}

const SENTIENT_STANDARD_STRESS_PENALTY_MULTIPLIER: f32 = 0.01;

#[derive(Component)]
pub struct LeftBehind;

pub fn apply_sentient_standard_stress_system(
    mut commands: Commands,
    standard: Option<Res<SentientStandard>>,
    metrics: Option<Res<ColonyMetrics>>,
    mut pop_query: Query<(Entity, &mut StressTracker, Option<&LeftBehind>), With<Pop>>,
) {
    if let (Some(standard), Some(metrics)) = (standard, metrics) {
        if !standard.active {
            for (entity, _, left_behind) in pop_query.iter_mut() {
                if left_behind.is_some() {
                    commands.entity(entity).remove::<LeftBehind>();
                }
            }
            return;
        }

        let deficit = standard.expected_luxury - metrics.current_luxury;

        if deficit > f32::EPSILON {
            let stress_penalty = deficit * SENTIENT_STANDARD_STRESS_PENALTY_MULTIPLIER;

            for (entity, mut tracker, left_behind) in pop_query.iter_mut() {
                if left_behind.is_none() {
                    commands.entity(entity).insert(LeftBehind);
                }
                tracker.accumulated_stress += stress_penalty;
            }
        } else {
            for (entity, _, left_behind) in pop_query.iter_mut() {
                if left_behind.is_some() {
                    commands.entity(entity).remove::<LeftBehind>();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::StressTracker;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems and resources
        world.insert_resource(SentientStandard {
            expected_luxury: 50.0,
            active: true,
        });
        world.insert_resource(ColonyMetrics {
            current_luxury: 30.0,
        });
        world
    }

    #[test]
    fn test_pop_gains_stress_when_below_standard() {
        let mut world = setup_world();

        let pop = world.spawn((Pop, StressTracker::default())).id();

        // Run the system that evaluates the Sentient Standard
        world
            .run_system_once(apply_sentient_standard_stress_system)
            .unwrap();
        world
            .run_system_once(apply_sentient_standard_stress_system)
            .unwrap();
        world
            .run_system_once(apply_sentient_standard_stress_system)
            .unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Luxury is 20 below standard, should generate proportional stress
        assert!(stress.accumulated_stress > f32::EPSILON);
        assert!(world.get::<LeftBehind>(pop).is_some());
    }

    #[test]
    fn test_pop_does_not_gain_stress_when_above_standard() {
        let mut world = setup_world();

        // Exceed the standard
        world.resource_mut::<ColonyMetrics>().current_luxury = 60.0;

        let pop = world.spawn((Pop, StressTracker::default())).id();

        world
            .run_system_once(apply_sentient_standard_stress_system)
            .unwrap();
        world
            .run_system_once(apply_sentient_standard_stress_system)
            .unwrap();
        world
            .run_system_once(apply_sentient_standard_stress_system)
            .unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Should not gain stress from Sentient Standard
        assert!((stress.accumulated_stress - 0.0).abs() < f32::EPSILON);
        assert!(world.get::<LeftBehind>(pop).is_none());
    }

    #[test]
    fn test_sentient_standard_inactive_causes_no_stress() {
        let mut world = setup_world();

        // Deactivate the standard (e.g., broadcast hasn't reached colony yet)
        world.resource_mut::<SentientStandard>().active = false;

        let pop = world.spawn((Pop, StressTracker::default())).id();

        world
            .run_system_once(apply_sentient_standard_stress_system)
            .unwrap();
        world
            .run_system_once(apply_sentient_standard_stress_system)
            .unwrap();
        world
            .run_system_once(apply_sentient_standard_stress_system)
            .unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Inactive standard shouldn't cause stress despite metrics being low
        assert!((stress.accumulated_stress - 0.0).abs() < f32::EPSILON);
        assert!(world.get::<LeftBehind>(pop).is_none());
    }
}
