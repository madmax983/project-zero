//! Integration tests for social tavern mechanics.

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::OccupiedTiles;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::lighting::{AmbientLight, LightMap};
    use scale::layer1::map::GridPosition;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::social::Tavern;
    use scale::layer1::terrain::{TerrainGrid, TerrainType};
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
        world.insert_resource(LightMap::new(10, 10));
        world.insert_resource(AmbientLight::default());
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
    fn test_social_seam_integration() {
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

        // Spawn a Pop at (0, 0) with low leisure
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 0.1, // Very bored
                    hunger: 0.8,
                    rest: 0.8,
                },
                PopAction::default(),
                UtilityWeights::default(),
            ))
            .id();

        // Run simulation until pop makes a decision (ticks_committed defaults to 0, so should decide immediately)
        // UtilityConfig evaluation_interval is 1.

        // Tick 1: Update timer (0 -> 1)
        run_simulation_tick(&mut world);
        // Tick 2: Evaluate (1 >= 1) -> Decide Socialize -> StartPlan
        run_simulation_tick(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Socialize,
            "Pop should decide to socialize"
        );

        // Run ticks to move to tavern. Distance is 5+5=10.
        // It takes ~10-12 ticks to travel.
        for _ in 0..15 {
            run_simulation_tick(&mut world);
        }

        // Pop should be at tavern
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);

        // Pop should be assigned to Tavern (THIS WILL FAIL currently)
        let tavern_comp = world.get::<Tavern>(tavern).unwrap();
        assert!(
            tavern_comp.visitors.contains(&pop),
            "Pop should be in tavern visitors list"
        );

        // Run more ticks to restore leisure
        run_simulation_tick(&mut world);

        // Leisure should have increased
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.1,
            "Leisure should increase after socializing"
        );
    }
}
