use crate::layer1::pop::{Job, Pop};
use crate::layer1::utility_types::AssignmentType;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SleepPermit {
    pub tier: PermitTier,
    pub allotted_hours: f32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PermitTier {
    Gold,
    Silver,
    Bronze,
}

#[derive(Component)]
pub struct FatigueTracker {
    pub current: f32,
}

#[allow(clippy::type_complexity)]
pub fn assign_sleep_permits_system(
    mut commands: Commands,
    pops: Query<(Entity, &Job), (With<Pop>, Without<SleepPermit>)>,
) {
    for (entity, job) in pops.iter() {
        let (permit, fatigue) = match job.job_type {
            AssignmentType::Administrator
            | AssignmentType::LibraryWorker
            | AssignmentType::ObservatoryWorker => (
                SleepPermit {
                    tier: PermitTier::Gold,
                    allotted_hours: 8.0,
                },
                FatigueTracker { current: 0.0 },
            ),
            AssignmentType::Sheriff | AssignmentType::Surgery => (
                SleepPermit {
                    tier: PermitTier::Silver,
                    allotted_hours: 5.0,
                },
                FatigueTracker { current: 0.0 },
            ),
            _ => (
                SleepPermit {
                    tier: PermitTier::Bronze,
                    allotted_hours: 2.0,
                },
                FatigueTracker { current: 0.0 },
            ),
        };
        commands.entity(entity).insert((permit, fatigue));
    }
}

#[allow(clippy::type_complexity)]
pub fn process_sleep_deprivation_system(
    mut commands: Commands,
    time: Res<bevy::time::Time>,
    mut pops: Query<
        (
            Entity,
            &SleepPermit,
            &mut FatigueTracker,
            &mut crate::layer1::stress::StressTracker,
        ),
        With<Pop>,
    >,
) {
    let delta = time.delta_secs();

    for (entity, permit, mut fatigue, mut stress) in pops.iter_mut() {
        // Fatigue increases faster for lower tier permits
        match permit.tier {
            PermitTier::Bronze => fatigue.current += 5.0 * delta,
            PermitTier::Silver => fatigue.current += 2.0 * delta,
            PermitTier::Gold => fatigue.current -= 1.0 * delta, // Rested
        }

        // High fatigue causes stress
        if fatigue.current > 80.0 {
            stress.accumulated_stress += 2.0 * delta;
        }

        // Extreme fatigue causes hallucinations
        if fatigue.current > 95.0 {
            commands.entity(entity).insert(crate::layer1::agriculture::gastronomy::Hallucinating { duration: 100 });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{Job, Pop};
    use crate::layer1::utility_types::AssignmentType;
    use bevy::prelude::*;

    #[test]
    fn test_sleep_permit_allocation_by_job_tier() {
        let mut app = App::new();

        let elite = app
            .world_mut()
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::Administrator,
                },
            ))
            .id();
        let mid = app
            .world_mut()
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::Sheriff,
                },
            ))
            .id();
        let worker = app
            .world_mut()
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        app.add_systems(Update, assign_sleep_permits_system);
        app.update();

        // Assert: Elite jobs get better permits and fatigue tracking is added
        let elite_permit = app.world().get::<SleepPermit>(elite).unwrap();
        let mid_permit = app.world().get::<SleepPermit>(mid).unwrap();
        let worker_permit = app.world().get::<SleepPermit>(worker).unwrap();

        assert_eq!(elite_permit.tier, PermitTier::Gold);
        assert_eq!(mid_permit.tier, PermitTier::Silver);
        assert_eq!(worker_permit.tier, PermitTier::Bronze);
        assert!(elite_permit.allotted_hours > worker_permit.allotted_hours);

        assert!(app.world().get::<FatigueTracker>(worker).is_some());
    }

    #[test]
    fn test_sleep_deprivation_increases_stress() {
        let mut app = App::new();
        // Needs TimePlugin for Res<Time>
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_sleep_deprivation_system);

        let worker = app
            .world_mut()
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::FarmWorker,
                },
                SleepPermit {
                    tier: PermitTier::Bronze,
                    allotted_hours: 2.0,
                },
                FatigueTracker { current: 95.0 }, // Very tired
                crate::layer1::stress::StressTracker {
                    accumulated_stress: 50.0,
                },
            ))
            .id();

        // Let's set the time manually by advancing it. We need two updates so time delta > 0
        app.update();
        app.world_mut()
            .resource_mut::<bevy::time::Time>()
            .advance_by(bevy::utils::Duration::from_secs(10));
        app.update();

        let stress = app
            .world()
            .get::<crate::layer1::stress::StressTracker>(worker)
            .unwrap();
        assert!(
            stress.accumulated_stress > 50.0,
            "Stress should increase, was {}",
            stress.accumulated_stress
        );
        assert!(
            app.world().get::<crate::layer1::agriculture::gastronomy::Hallucinating>(worker).is_some(),
            "Should be hallucinating"
        );
    }
}
