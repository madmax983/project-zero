#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::combat::Drafted;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::factions::{FactionData, FactionId, FactionMember, FactionState, Factions};
    use crate::layer1::farm::Farm;
    use crate::layer1::law::penal::PenalLabor;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::Tavern;
    use crate::layer1::unrest::{MentalBreakType, MentalState};
    use crate::layer1::utility_ai::{ActionType, PopAction, UtilityConfig};
    use crate::layer1::utility_types::UtilityWeights;
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        crate::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));
        world
    }

    #[test]
    fn test_drafted_pop_ignores_critical_hunger() {
        let mut world = setup_world();

        // Spawn a starving pop, but DRAFTED
        let pop = world
            .spawn((
                Pop,
                Drafted,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1, // Starving!
                    rest: 0.8,
                    leisure: 0.8,
                    hygiene: 0.8,
                    oxygen: 100.0,
                },
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    ticks_committed: 10,
                    ..Default::default()
                },
            ))
            .id();

        // Available Farm (Food)
        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 1, y: 0 },
            Farm::default(),
        ));

        // Evaluate
        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        // Should NOT be SatisfyHunger because Drafted overrides normal needs
        assert_ne!(
            action.current,
            ActionType::SatisfyHunger,
            "Drafted pop should ignore hunger"
        );
    }

    #[test]
    fn test_striking_pop_ignores_work_designations() {
        let mut world = setup_world();

        // Setup Factions Resource with a Striking Faction
        let mut factions = Factions::default();
        // Use MinersGuild for the test
        factions.map.insert(
            FactionId::MinersGuild,
            FactionData {
                name: "Union".to_string(),
                state: FactionState::Striking, // Strike!
                ..Default::default()
            },
        );
        world.insert_resource(factions);

        // Spawn Striking Worker
        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    ticks_committed: 10,
                    ..Default::default()
                },
            ))
            .id();

        // High priority Work Designation
        world.spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 1, y: 0 },
        ));

        // Evaluate
        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_ne!(
            action.current,
            ActionType::Work,
            "Striking pop should refuse to work"
        );
    }

    #[test]
    fn test_penal_labor_ignores_socializing() {
        let mut world = setup_world();

        // Spawn Inmate with PenalLabor and low leisure
        let pop = world
            .spawn((
                Pop,
                PenalLabor::default(),
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.8,
                    rest: 0.8,
                    leisure: 0.1,
                    hygiene: 0.8,
                    oxygen: 100.0, // Very bored
                },
                UtilityWeights::default(),
                PopAction {
                    ticks_committed: 10,
                    ..Default::default()
                },
            ))
            .id();

        // Available Tavern
        world.spawn((Tavern::default(), GridPosition { x: 1, y: 0 }));

        // Evaluate
        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_ne!(
            action.current,
            ActionType::Socialize,
            "Penal Labor should prevent socializing"
        );
    }

    #[test]
    fn test_mental_break_overrides_drafted() {
        let mut world = setup_world();

        // Spawn Drafted Pop with Mental Break (Vandalize)
        let pop = world
            .spawn((
                Pop,
                Drafted,
                MentalState::Broken(MentalBreakType::Vandalize), // Break!
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    ticks_committed: 10,
                    ..Default::default()
                },
            ))
            .id();

        // Add a target for Vandalize (Structure)
        world.spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 1, y: 0 },
            crate::layer1::structure::Structure {
                current_hp: 100.0,
                max_hp: 100.0,
            },
        ));

        // Enemy present (would normally trigger Fight)
        world.spawn((
            crate::layer1::fauna::Fauna::default(),
            GridPosition { x: 5, y: 5 },
            crate::layer1::health::Health::default(),
        ));

        // Evaluate
        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Vandalize,
            "Mental Break should override Drafted state"
        );
        assert_ne!(
            action.current,
            ActionType::Fight,
            "Should not fight while having a mental break"
        );
    }

    #[test]
    fn test_penal_labor_allows_eating() {
        // Penal labor restricts fun, but not survival
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                PenalLabor::default(),
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1, // Starving
                    rest: 0.8,
                    leisure: 0.8,
                    hygiene: 0.8,
                    oxygen: 100.0,
                },
                UtilityWeights::default(),
                PopAction {
                    ticks_committed: 10,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 1, y: 0 },
            Farm::default(),
        ));

        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::SatisfyHunger,
            "Penal Labor must still allow eating"
        );
    }

    #[test]
    fn test_evaluate_actions_chooses_shower_when_dirty_and_has_water() {
        let mut world = setup_world();
        world
            .resource_mut::<crate::layer1::resources::ColonyResources>()
            .water = 10.0;

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.8,
                    rest: 0.8,
                    leisure: 0.8,
                    hygiene: 0.1,
                    oxygen: 100.0, // Dirty!
                },
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    ticks_committed: 10,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::Shower,
            },
            GridPosition { x: 1, y: 0 },
        ));

        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::UseShower,
            "Pop should choose to shower when dirty and water is available"
        );
    }

    #[test]
    fn test_evaluate_actions_ignores_shower_when_no_water() {
        let mut world = setup_world();
        // default water is 0.0

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.8,
                    rest: 0.8,
                    leisure: 0.8,
                    hygiene: 0.1,
                    oxygen: 100.0, // Dirty!
                },
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    ticks_committed: 10,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::Shower,
            },
            GridPosition { x: 1, y: 0 },
        ));

        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Pop should not choose to shower when there is no water"
        );
    }

    #[test]
    fn test_striking_pop_eats() {
        let mut world = setup_world();

        let mut factions = Factions::default();
        factions.map.insert(
            FactionId::MinersGuild,
            FactionData {
                name: "Union".to_string(),
                state: FactionState::Striking,
                ..Default::default()
            },
        );
        world.insert_resource(factions);

        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1,
                    rest: 0.8,
                    leisure: 0.8,
                    hygiene: 0.8,
                    oxygen: 100.0,
                },
                UtilityWeights::default(),
                PopAction {
                    ticks_committed: 10,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 1, y: 0 },
            Farm::default(),
        ));

        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::SatisfyHunger,
            "Striking pop should still eat"
        );
    }
}
