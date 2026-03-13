#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::actions::{AssignedTo, AssignmentType};
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::cryo_shock::CryoShock;
    use scale::layer1::health::Health;
    use scale::layer1::map::GridPosition;
    use scale::layer1::medical::{healing_system, Hospital};
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::utility_ai::{evaluate_actions_system, UtilityConfig};
    use scale::layer1::utility_types::ActionType;
    use scale::layer1::utility_types::UtilityWeights;
    use scale::layer1::PopAction;

    #[test]
    fn test_utility_ai_assigns_medical_care_for_cryo_shock() {
        scale::setup::init_task_pools();
        let mut world = World::new();

        // Required resources for UtilityAI
        world.insert_resource(UtilityConfig {
            switch_threshold: 0.1,
            ..Default::default()
        });
        world.insert_resource(scale::shared::time::SimulationTime::default());
        world.insert_resource(scale::layer1::resources::ColonyResources::default());
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::taboo::TabooState::default());
        world.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10));

        // Spawn a Hospital
        world.spawn((
            Building {
                building_type: BuildingType::Hospital,
            },
            Hospital::default(),
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn a Pop with full health but WITH CryoShock
        let pop = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                }, // Full health
                CryoShock {
                    duration_ticks: 1000,
                    severity: 0.5,
                },
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.1,
                    ticks_committed: 10,
                },
            ))
            .id();

        // Run AI evaluation
        evaluate_actions_system(&mut world);

        // Assert the pop decided to SeekMedicalCare because of CryoShock
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::SeekMedicalCare);
    }

    #[test]
    fn test_healing_system_accelerates_cryo_shock_decay() {
        let mut world = World::new();

        // Spawn Hospital
        let hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital::default(), // 5.0 capacity
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Spawn Patient with CryoShock
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
                    duration_ticks: 1000,
                    severity: 0.5,
                },
            ))
            .id();

        // Run healing system
        let mut schedule = Schedule::default();
        schedule.add_systems(healing_system);
        schedule.run(&mut world);

        // Assert CryoShock duration is aggressively reduced
        let shock = world.get::<CryoShock>(patient).unwrap();
        assert!(
            shock.duration_ticks <= 980,
            "CryoShock duration should decay by at least 20 ticks in hospital"
        );
    }
}
