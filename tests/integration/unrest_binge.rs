//! Integration tests for Unrest (Binge) behavior.

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{OccupiedTiles, Building, BuildingType};
    use scale::layer1::map::GridPosition;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::social::Tavern;
    use scale::layer1::stockpile::Stockpile;
    use scale::layer1::terrain::{TerrainGrid, TerrainType};
    use scale::layer1::unrest::{MentalBreakType, MentalState};
    use scale::layer1::utility_ai::{ActionType, PopAction, UtilityConfig, UtilityWeights};
    use scale::shared::state::GameState;
    use scale::shared::time::SimulationTime;
    use scale::simulation::run_simulation_tick;

    fn setup_world() -> World {
        let mut world = World::new();
        // Required resources
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(GameState::Running);
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(scale::shared::log::MessageLog::default());
        world.insert_resource(scale::layer1::seasons::SeasonState::default());
        world.insert_resource(scale::layer1::utility_ai::ColonyMemory::default());
        world.insert_resource(scale::layer1::chronicle::BuildingTracker::default());
        world.insert_resource(scale::layer1::chronicle::Chronicle::default());
        world.insert_resource(scale::shared::narrative::NarrativeGenerator::from_embedded());
        world.insert_resource(scale::shared::colony::ColonyName::default());
        world.insert_resource(scale::layer1::beauty::BeautyGrid::new(10, 10));
        world.insert_resource(scale::layer1::acoustic::NoiseMap::new(10, 10));
        world.insert_resource(scale::layer1::notifications::NotificationQueue::default());
        world.insert_resource(scale::layer1::lighting::LightMap::new(10, 10));
        world.insert_resource(scale::layer1::lighting::AmbientLight::default());
        world.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10));
        world.insert_resource(scale::layer1::visitor::VisitorSource::default());
        world.insert_resource(scale::layer1::vermin::VerminState::default());
        world.insert_resource(scale::layer1::edicts::ColonyPolicies::default());
        world.insert_resource(scale::layer1::trade::MerchantState::default());
        world.insert_resource(scale::layer1::atmosphere::AtmosphereGrid::new(10, 10));
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.init_resource::<Events<scale::layer1::chronicle::AddChronicleEvent>>();
        world.init_resource::<Events<scale::layer1::social::AffinityChange>>();
        world.init_resource::<Events<scale::layer1::pop::PopDied>>();

        world
    }

    #[test]
    fn test_binge_finds_target_tavern() {
        let mut world = setup_world();

        // Spawn a Tavern at (5, 5)
        let tavern = world
            .spawn((
                Building {
                    building_type: BuildingType::Tavern,
                },
                GridPosition { x: 5, y: 5 },
                Tavern::default(),
            ))
            .id();

        // Spawn a Pop with Binge mental break
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1,
                    rest: 0.1,
                    leisure: 0.1,
                },
                MentalState::Broken(MentalBreakType::Binge),
                PopAction::default(),
                UtilityWeights::default(),
            ))
            .id();

        // Run simulation
        // Tick 1: Update timer
        run_simulation_tick(&mut world);
        // Tick 2: Evaluate actions
        run_simulation_tick(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Binge,
            "Pop should decide to Binge"
        );

        // Check MovementTarget
        let mt = world.get::<scale::layer1::execution::MovementTarget>(pop);
        assert!(mt.is_some(), "Pop should have a movement target");
        assert_eq!(mt.unwrap().target_entity, tavern, "Pop should target Tavern");
    }

    #[test]
    fn test_binge_finds_target_stockpile() {
        let mut world = setup_world();

        // Spawn a Stockpile at (5, 5)
        let stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                GridPosition { x: 5, y: 5 },
                Stockpile::default(),
            ))
            .id();

        // Spawn a Pop with Binge mental break
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1,
                    rest: 0.1,
                    leisure: 0.1,
                },
                MentalState::Broken(MentalBreakType::Binge),
                PopAction::default(),
                UtilityWeights::default(),
            ))
            .id();

        // Run simulation
        run_simulation_tick(&mut world);
        run_simulation_tick(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Binge,
            "Pop should decide to Binge"
        );

        // Check MovementTarget
        let mt = world.get::<scale::layer1::execution::MovementTarget>(pop);
        assert!(mt.is_some(), "Pop should have a movement target");
        assert_eq!(mt.unwrap().target_entity, stockpile, "Pop should target Stockpile");
    }

    #[test]
    fn test_binge_consumes_resources() {
        let mut world = setup_world();

        // Setup resources
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.food = 50.0;
            res.max_food = 100.0;
        }

        // Spawn a Tavern
        let _tavern = world
            .spawn((
                Building {
                    building_type: BuildingType::Tavern,
                },
                GridPosition { x: 5, y: 5 },
                Tavern::default(),
            ))
            .id();

        // Spawn a Pop adjacent to Tavern with Binge
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 4, y: 5 }, // Adjacent
                Needs {
                    hunger: 0.1,
                    rest: 0.1,
                    leisure: 0.1,
                },
                MentalState::Broken(MentalBreakType::Binge),
                PopAction::default(),
                UtilityWeights::default(),
            ))
            .id();

        // Run simulation to trigger evaluation and movement
        // Tick 1: Update timer
        run_simulation_tick(&mut world);
        // Tick 2: Evaluate -> Binge -> StartPlan -> MovementTarget
        run_simulation_tick(&mut world);
        // Tick 3: Move (arrives at 5,5 or adjacent) -> Arrive -> Binge execution starts
        run_simulation_tick(&mut world);

        // Pop should be AtTarget
        // Note: movement_system runs, sees dist=1 (4,5 to 5,5).
        // Since it's Binge (not Work/Repair), it might try to move ONTO the Tavern.
        // Taverns are walkable? Buildings usually are NOT walkable unless specified.
        // Let's assume Tavern is NOT walkable.
        // If not walkable, movement_system checks if adjacent for Work/Repair.
        // Binge is not Work/Repair.
        // So Binge might need to be treated like Work/Repair/Socialize?
        // Socialize enters the tavern (AssignedTo).
        // If Binge behaves like Socialize, it should enter.
        // But Socialize uses `handle_socialize` which assigns to tavern.
        // Binge consumes resources.
        // If I make Binge just stand adjacent (like Vandalize), it's easier.
        // Vandalize uses `vandalize_execution_system` which works on `AtTarget`.
        // `movement_system` sets `AtTarget` if adjacent for Work/Repair.
        // It does NOT do so for Vandalize/Binge unless I add it.
        // Wait, `vandalize_execution_system` iterates `AtTarget`.
        // Does `movement_system` put Vandals at target?
        // Let's check `movement_system` in `execution.rs`.

        // Logic check in execution.rs:
        // if action == ActionType::Work || action == ActionType::Repair { ... check adjacent ... }
        // It does NOT check Vandalize.
        // So Vandals must walk ONTO the target?
        // If target is building, it's unwalkable.
        // So Vandals might get stuck?
        // Ah, `perform_vandalize_logic` test `test_vandalize_integration_damages_building` passed?
        // Wait, I didn't run that test. It was in `unrest.rs` test module I read.
        // Let me check `unrest.rs` test again.
        // It spawns pop at (0,0), building at (1,0).
        // Calls `evaluate_actions_system`.
        // Calls `process_start_plan_system`.
        // Calls `movement_system`.
        // Then checks `AtTarget`.
        // If building is unwalkable, `movement_system` fails to move to (1,0).
        // Unless building is walkable?
        // `Building` component doesn't imply unwalkable. `TerrainType` does.
        // Buildings are usually placed on terrain.
        // Does `Building` make terrain unwalkable?
        // `building_placement` system usually updates `OccupiedTiles`.
        // `is_walkable_terrain` checks `TerrainGrid`.
        // It does NOT check `OccupiedTiles` or `Building`.
        // So buildings are walkable unless terrain is set to something else?
        // `TerrainType::Wall`?
        // Usually buildings are just entities.
        // So Vandals CAN walk onto buildings.
        // So Binge pops CAN walk onto Tavern.

        let food_before = world.resource::<ColonyResources>().food;

        // Tick 4: Execute Binge logic (Probabilistic 10%)
        // Loop until consumption happens or timeout
        let mut consumed = false;
        let mut logged = false;

        for _ in 0..100 {
            run_simulation_tick(&mut world);
            let food_current = world.resource::<ColonyResources>().food;
            if food_current < food_before {
                consumed = true;
            }

            let log = world.resource::<scale::shared::log::MessageLog>();
            if log.messages.iter().any(|m| m.text.contains("binging")) {
                logged = true;
            }

            if consumed && logged {
                break;
            }
        }

        assert!(consumed, "Binge should consume food eventually");
        // Log check (Probabilistic 1% of total ticks, might be flaky if not checked properly)
        // With 100 ticks, 1% chance per tick -> ~63% chance to happen at least once.
        // This is still flaky.
        // Maybe I shouldn't assert log presence in integration test if it's rare.
        // Or I should accept that it might not log.
        // But I want to verify it CAN log.
        // Let's rely on unit tests for rare logic? But unrest.rs has unit tests which I modified.
        // Wait, perform_binge_logic in unit test uses thread_rng too.
        // I should probably make perform_binge_logic take a seed or force it?
        // No easy way.
        // I will assert log ONLY if consumed (which implies logic ran).
        // Actually, if consumed is true, logic ran.
        // If logged is false, it just didn't hit the 10% log chance.
        // I will relax the log assertion or skip it.
        // Or just assert consumed.
    }
}
