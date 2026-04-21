#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::combat::{execute_attack, AttackProperties, CombatState, Drafted, Weapon};
    use scale::layer1::flora::{flora_spread_system, Flora, FloraType};
    use scale::layer1::health::Health;
    use scale::layer1::items::Equipment;
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::terrain::{TerrainGrid, TerrainType};
    use scale::layer1::utility_ai::{
        evaluate_actions_system, ActionType, PopAction, UtilityConfig,
    };
    use scale::layer1::utility_types::UtilityWeights;

    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(scale::shared::time::SimulationTime::default());
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(scale::layer1::map::ScreenShake::default());

        world.insert_resource(scale::layer1::resources::ColonyResources::default());
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::taboo::TabooState::default());
        world.insert_resource(scale::layer1::factions::Factions::default());
        world.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10));

        // Terrain for flora spread
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        world
    }

    #[test]
    fn test_drafted_pop_targets_flora() {
        let mut world = setup_world();

        let weapon = world
            .spawn(Weapon {
                properties: AttackProperties {
                    damage: 10.0,
                    range: 5.0,
                    cooldown: 10,
                    accuracy: 1.0,
                },
            })
            .id();

        let pop = world
            .spawn((
                Pop,
                Drafted,
                Equipment {
                    weapon: Some(weapon),
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
                scale::layer1::needs::Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.0,
                    ticks_committed: 100,
                },
                CombatState::default(),
            ))
            .id();

        let flora = world
            .spawn((
                Flora {
                    flora_type: FloraType::XenoMoss,
                    ..Default::default()
                },
                GridPosition { x: 2, y: 0 },
                Health::default(), // Ensure valid target
            ))
            .id();

        evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Fight, "Action should be Fight");

        let start_plan = world.get::<scale::layer1::utility_types::StartPlan>(pop);
        assert_eq!(start_plan.unwrap().target, Some(flora));
    }

    #[test]
    fn test_drafted_pop_damages_flora() {
        let mut world = setup_world();

        let weapon = world
            .spawn(Weapon {
                properties: AttackProperties {
                    damage: 10.0,
                    range: 5.0,
                    cooldown: 0,
                    accuracy: 1.0,
                },
            })
            .id();

        let pop = world
            .spawn((
                Pop,
                Drafted,
                Equipment {
                    weapon: Some(weapon),
                    ..Default::default()
                },
                CombatState::default(),
            ))
            .id();

        // Spawn Flora WITH Health (simulating correct spawning)
        let flora = world
            .spawn((
                Flora::default(),
                GridPosition { x: 0, y: 0 },
                Health {
                    current: 20.0,
                    max: 20.0,
                    conditions: Vec::new(),
                },
            ))
            .id();

        // Execute Attack
        execute_attack(&mut world, pop, flora);

        // Assert Damage
        let health = world.get::<Health>(flora).expect("Flora must have Health");
        assert!(health.current < 20.0, "Flora should take damage");
    }

    #[test]
    fn test_flora_spread_creates_valid_target() {
        let mut world = setup_world();

        // 1. Spawn "Patient Zero" Flora (ready to spread)
        let patient_zero = world
            .spawn((
                Flora {
                    growth_timer: 0,
                    spread_chance: 1.0, // Force spread
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
                Health::default(),
            ))
            .id();

        // 2. Run Spread System
        // flora_spread_system requires Commands, Query, Terrain, Query<Other>
        // We can run it via RunSystemOnce or Schedule
        let mut schedule = Schedule::default();
        schedule.add_systems(flora_spread_system);
        schedule.run(&mut world);

        // 3. Find the NEW Flora
        // Should be at a neighbor of 5,5
        let mut flora_query = world.query::<(Entity, &GridPosition, &Flora, Option<&Health>)>();
        let mut new_flora = None;

        for (e, pos, _, health_opt) in flora_query.iter(&world) {
            if e != patient_zero {
                new_flora = Some((e, *pos, health_opt));
            }
        }

        assert!(new_flora.is_some(), "Flora should have spread");
        let (new_entity, new_pos, health_opt) = new_flora.unwrap();

        // 4. Verify Health Presence (The Fix)
        assert!(health_opt.is_some(), "New Flora MUST have Health component");

        // 5. Verify Targeting
        let weapon = world
            .spawn(Weapon {
                properties: AttackProperties {
                    damage: 10.0,
                    range: 10.0,
                    cooldown: 0,
                    accuracy: 1.0,
                },
            })
            .id();

        let pop = world
            .spawn((
                Pop,
                Drafted,
                Equipment {
                    weapon: Some(weapon),
                    ..Default::default()
                },
                // Place pop near new flora
                GridPosition {
                    x: new_pos.x,
                    y: new_pos.y,
                },
                scale::layer1::needs::Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    ticks_committed: 100,
                    ..Default::default()
                },
                CombatState::default(),
            ))
            .id();

        evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        let plan = world.get::<scale::layer1::utility_types::StartPlan>(pop);

        assert_eq!(action.current, ActionType::Fight, "Should target new flora");
        assert_eq!(plan.unwrap().target, Some(new_entity));
    }
}
