#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::factions::{FactionData, FactionId, FactionMember, FactionState, Factions};
    use scale::layer1::farm::{Farm, produce_food_system};
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::skills::{SkillType, Skills};
    use scale::layer1::utility_ai::{ActionType, PopAction};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(Factions::default());
        // Initialize Factions
        world.resource_mut::<Factions>().initialize();
        world
    }

    #[test]
    fn striking_farmer_produces_no_food() {
        let mut world = setup_world();

        // 1. Set Farmers Guild to Striking
        if let Some(mut factions) = world.get_resource_mut::<Factions>() {
            if let Some(guild) = factions.map.get_mut(&FactionId::FarmersGuild) {
                guild.state = FactionState::Striking;
            }
        }

        // 2. Spawn a Farm
        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // 3. Spawn a Striking Farmer working at the farm
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Farming, 100.0);

        world.spawn((
            Pop,
            skills,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
            FactionMember {
                faction_id: Some(FactionId::FarmersGuild),
            },
        ));

        // 4. Run produce_food_system
        let mut schedule = Schedule::default();
        schedule.add_systems(produce_food_system);
        schedule.run(&mut world);

        // 5. Assert NO food produced
        let resources = world.resource::<ColonyResources>();
        // Default food is 10.0
        assert!(
            (resources.food - 10.0).abs() < f32::EPSILON,
            "Striking farmer should produce NO food, but food is {}",
            resources.food
        );
    }

    #[test]
    fn striking_pop_refuses_work_assignment() {
        let mut world = setup_world();
        scale::setup::init_task_pools(); // Required for parallel execution in utility_ai

        // 1. Set Farmers Guild to Striking
        if let Some(mut factions) = world.get_resource_mut::<Factions>() {
            if let Some(guild) = factions.map.get_mut(&FactionId::FarmersGuild) {
                guild.state = FactionState::Striking;
            }
        }

        // 2. Spawn a Farm
        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // 3. Spawn a Farmer
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Farming, 100.0);

        let pop = world
            .spawn((
                Pop,
                skills,
                GridPosition { x: 5, y: 5 },
                scale::layer1::needs::Needs {
                    hunger: 1.0,
                    rest: 1.0,
                    leisure: 1.0,
                }, // Fully satisfied needs
                scale::layer1::utility_ai::UtilityWeights::default(),
                PopAction {
                    ticks_committed: 100, // Ready to evaluate
                    ..Default::default()
                },
                FactionMember {
                    faction_id: Some(FactionId::FarmersGuild),
                },
            ))
            .id();

        // 4. Run evaluate_actions_system
        world.insert_resource(scale::shared::time::SimulationTime::default());
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::taboo::TabooState::default());
        world.insert_resource(scale::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(scale::layer1::structural_integrity::RoofGrid::new(10, 10)); // Required by CandidateQueries

        let mut schedule = Schedule::default();
        schedule.add_systems(scale::layer1::utility_ai::evaluate_actions_system);
        schedule.run(&mut world);

        // 5. Assert Action is NOT Farm
        let action = world.get::<PopAction>(pop).unwrap();
        assert_ne!(
            action.current,
            ActionType::Farm,
            "Striking pop should not choose Farm"
        );
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Striking pop should be Idle"
        );
    }
}
