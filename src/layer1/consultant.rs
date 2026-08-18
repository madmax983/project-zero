use bevy::prelude::*;

#[derive(Component)]
pub struct ConsultantMarker;

#[derive(Resource, Default)]
pub struct JobPriorities {
    pub prioritize_safety: bool,
    pub maximize_profit: bool,
}

#[derive(Component)]
pub struct ConsultantWorker;

#[derive(Component)]
pub struct ConsultantEfficiency(pub f32);

#[derive(Component)]
pub struct ConsultantStress(pub f32);

#[derive(Component)]
pub struct BuffRadius(pub f32);

#[derive(Component)]
pub struct Reactor;

#[derive(Component)]
pub struct MaintenanceLevel(pub f32);

#[derive(Component)]
pub struct CriticalCondition(pub bool);

pub fn apply_consultant_override(
    consultants: Query<(), With<ConsultantMarker>>,
    mut priorities: ResMut<JobPriorities>,
) {
    if !consultants.is_empty() {
        priorities.prioritize_safety = false;
        priorities.maximize_profit = true;
    }
}

pub fn consultant_worker_impact(
    consultants: Query<(), With<ConsultantMarker>>,
    mut workers: Query<(&mut ConsultantEfficiency, &mut ConsultantStress), With<ConsultantWorker>>,
    time: Res<Time>,
) {
    if !consultants.is_empty() {
        for (mut efficiency, mut stress) in workers.iter_mut() {
            efficiency.0 = 2.0; // Massive buff
            stress.0 += 1.0 * time.delta_secs(); // High stress
        }
    }
}

pub fn consultant_hazard_escalation(
    consultants: Query<(), With<ConsultantMarker>>,
    mut hazards: Query<(&MaintenanceLevel, &mut CriticalCondition), With<Reactor>>,
) {
    if !consultants.is_empty() {
        for (maintenance, mut condition) in hazards.iter_mut() {
            if maintenance.0 < 0.5 {
                condition.0 = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consultant_spawn_triggers_profit_override() {
        let mut app = App::new();
        // Setup initial state: Normal priorities
        app.insert_resource(JobPriorities {
            prioritize_safety: true,
            ..default()
        });

        // Spawn Consultant
        app.world_mut()
            .spawn((ConsultantMarker, Name::new("The Fixer")));

        // Run the system
        app.add_systems(Update, apply_consultant_override);
        app.update();

        // Assert: Priorities changed to Maximize Profit
        let priorities = app.world().resource::<JobPriorities>();
        assert!(!priorities.prioritize_safety);
        assert!(priorities.maximize_profit);
    }

    #[test]
    fn test_consultant_buffs_efficiency_but_increases_stress() {
        let mut app = App::new();
        let worker_entity = app
            .world_mut()
            .spawn((
                ConsultantWorker,
                ConsultantEfficiency(1.0),
                ConsultantStress(0.0),
            ))
            .id();

        app.insert_resource(Time::new_with(()));

        // Spawn Consultant in the same sector/global
        app.world_mut().spawn((ConsultantMarker, BuffRadius(50.0)));

        app.add_systems(Update, consultant_worker_impact);

        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(1));

        app.update();

        // Assert: Efficiency increased massively, but stress went up
        let efficiency = app
            .world()
            .get::<ConsultantEfficiency>(worker_entity)
            .unwrap();
        let stress = app.world().get::<ConsultantStress>(worker_entity).unwrap();

        assert!(
            efficiency.0 > 1.5,
            "Efficiency should receive a massive buff"
        );
        assert!(stress.0 > 0.5, "Stress should increase significantly");
    }

    #[test]
    fn test_consultant_causes_critical_reactor_failure() {
        let mut app = App::new();
        let reactor_entity = app
            .world_mut()
            .spawn((
                Reactor,
                MaintenanceLevel(0.0), // Low maintenance
                CriticalCondition(false),
            ))
            .id();

        // With consultant present, low maintenance leads to fast failure
        app.world_mut().spawn(ConsultantMarker);

        app.add_systems(Update, consultant_hazard_escalation);
        app.update();

        let condition = app
            .world()
            .get::<CriticalCondition>(reactor_entity)
            .unwrap();
        assert!(
            condition.0,
            "Reactor should reach critical condition under Consultant's watch"
        );
    }
}
