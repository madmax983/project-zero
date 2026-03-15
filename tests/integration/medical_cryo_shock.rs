#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::actions::{AssignedTo, AssignmentType};
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::cryo_shock::CryoShock;
    use scale::layer1::day_night::DayNightCycle;
    use scale::layer1::health::Health;
    use scale::layer1::map::GridPosition;
    use scale::layer1::medical::{healing_system, Hospital, MedicalPolicy};
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::stress::StressTracker;
    use scale::layer1::taboo::TabooState;
    use scale::layer1::utility_ai::{evaluate_actions_system, update_action_timer_system};
    use scale::layer1::utility_types::{ActionType, PopAction};

    #[test]
    fn test_healing_system_cures_cryo_shock() {
        let mut world = World::new();
        // Setup Hospital
        let hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital::default(),
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Spawn Patient with CryoShock (and full HP)
        let patient = world
            .spawn((
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                AssignedTo {
                    entity: hospital,
                    assignment_type: AssignmentType::Patient,
                },
                CryoShock {
                    duration_ticks: 100,
                    severity: 0.5,
                },
            ))
            .id();

        // Run healing system
        let mut schedule = Schedule::default();
        schedule.add_systems(healing_system);
        schedule.run(&mut world);

        // Assert CryoShock is reduced
        let shock = world.get::<CryoShock>(patient).unwrap();
        assert!(
            shock.duration_ticks < 100,
            "CryoShock duration should decrease"
        );
        // Ensure effective treatment (reduces by 10)
        assert_eq!(
            shock.duration_ticks, 90,
            "Hospital should reduce duration by 10"
        );
    }

    #[test]
    fn test_cryo_shock_pop_seeks_medical_care() {
        let mut app = bevy_app::App::new();

        let mut world = &mut app.world_mut();
        world.insert_resource(MedicalPolicy::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(TabooState::default());
        world.insert_resource(DayNightCycle::default());
        world.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10));
        world.insert_resource(scale::layer1::utility_types::UtilityConfig::default());
        world.insert_resource(scale::layer1::terrain::generate_terrain(10, 10));
        world.insert_resource(bevy_time::Time::<()>::default());

        // Setup Hospital
        let hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital::default(),
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Spawn Patient with CryoShock (and full HP)
        let patient = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                Needs::default(),
                StressTracker::default(),
                scale::layer1::utility_types::UtilityWeights::default(),
                PopAction::default(),
                GridPosition { x: 0, y: 0 },
                CryoShock {
                    duration_ticks: 100,
                    severity: 0.5,
                },
            ))
            .id();

        // Run Utility AI
        let mut schedule = Schedule::default();
        schedule.add_systems((update_action_timer_system, evaluate_actions_system).chain());
        schedule.run(world);

        // Assert Pop is seeking medical care
        let plan = world.get::<scale::layer1::utility_types::StartPlan>(patient);
        if let Some(plan) = plan {
            assert_eq!(
                plan.action,
                ActionType::SeekMedicalCare,
                "Pop with CryoShock should seek medical care"
            );
            assert_eq!(
                plan.target,
                Some(hospital),
                "Pop should target the hospital"
            );
        } else {
            let action = world.get::<PopAction>(patient).unwrap();
            assert_eq!(
                action.current,
                ActionType::SeekMedicalCare,
                "Pop with CryoShock should seek medical care"
            );
        }
    }
}
