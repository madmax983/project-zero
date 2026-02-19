#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use scale::layer1::GridPosition;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::execution::{AtTarget, MovementTarget, arrival_handler_system};
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use scale::layer1::stockpile::Stockpile;
    use scale::layer1::structural_integrity::StructureCollapsed;
    use scale::layer1::utility_ai::{ActionType, PopAction};
    use scale::shared::time::SimulationTime;
    use scale::simulation::{SimulationSchedule, build_simulation_schedule};

    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(scale::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![scale::layer1::terrain::TerrainType::Grass; 100],
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(scale::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::seasons::SeasonState::default());
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world.insert_resource(scale::shared::log::MessageLog::default());
        world.insert_resource(scale::layer1::chronicle::Chronicle::default());
        world.insert_resource(scale::layer1::chronicle::BuildingTracker::default());
        world.insert_resource(scale::layer1::erosion::ErosionGrid::new(10, 10));
        world.insert_resource(scale::layer1::water::WaterGrid::new(10, 10));
        world.insert_resource(scale::layer1::tech::TechState::default());
        world.insert_resource(scale::layer1::beauty::BeautyGrid::new(10, 10));
        world.insert_resource(scale::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10));
        world.insert_resource(scale::layer1::acoustic::NoiseMap::new(10, 10));
        world.insert_resource(scale::layer1::lighting::LightMap::new(10, 10));
        world.insert_resource(scale::layer1::lighting::AmbientLight::default());
        world.insert_resource(scale::layer1::atmosphere::AtmosphereGrid::new(10, 10));
        world.insert_resource(scale::layer1::pressure::PressureGrid::new(10, 10));
        world.insert_resource(scale::layer1::notifications::NotificationQueue::default());
        world.insert_resource(scale::layer1::trade::MerchantState::default());
        world.insert_resource(scale::layer1::prototyping::BuildingMastery::default());
        world.insert_resource(scale::layer1::vermin::VerminState::default());
        world.insert_resource(scale::layer1::edicts::ColonyPolicies::default());
        world.init_resource::<Events<scale::layer1::chronicle::AddChronicleEvent>>();
        world.init_resource::<Events<scale::layer1::social::AffinityChange>>();
        world.init_resource::<Events<scale::layer1::social::FavorChange>>();
        world.init_resource::<Events<scale::layer1::medical::PatientTreated>>();
        world.init_resource::<Events<scale::layer1::hazards::AmputationEvent>>();
        world.init_resource::<Events<scale::layer1::DeathEvent>>();
        world.init_resource::<Events<scale::layer1::pop::PopDied>>();
        world.insert_resource(scale::layer1::geology::SeismicGrid::new(10, 10));
        world.init_resource::<Events<StructureCollapsed>>();
        world.init_resource::<Events<scale::layer1::heirloom::RetrogradeEngineeringEvent>>();
        world.init_resource::<Events<scale::layer1::energy::GridOverloadEvent>>();
        world.init_resource::<Events<scale::layer1::geology::GeologicalEvent>>();
        world.init_resource::<Events<scale::layer1::society::InvestigationEvent>>();
        world.init_resource::<Events<scale::layer1::society::SuppressSocietyEvent>>();
        world.insert_resource(scale::layer1::social_mimicry::Trend::default());
        world.insert_resource(scale::layer1::visitor::VisitorSource::default());
        world.insert_resource(scale::layer1::weather::WeatherState::default());
        world.insert_resource(scale::layer1::quirks::PlanetaryTraits::default());
        world.insert_resource(scale::layer1::factions::Factions::default());
        world.insert_resource(scale::layer1::taboo::TabooState::default());
        world.insert_resource(scale::layer1::map::ScreenShake::default());
        world.insert_resource(scale::layer1::inspector::InspectorSource::default());
        world.insert_resource(scale::layer1::tech_envy::TechEnvyConfig::default());
        world.init_resource::<scale::layer1::social::old_guard::Demographics>();
        world.insert_resource(scale::layer1::building::BuildingMap::default());

        let generator = scale::shared::narrative::NarrativeGenerator::from_embedded();
        world.insert_resource(scale::shared::colony::ColonyName {
            name: "Test Colony".to_string(),
        });
        world.insert_resource(generator);

        // Initialize Schedules resource
        world.insert_resource(Schedules::default());
        world
    }

    #[test]
    fn test_arrival_handler_clobbers_hauling() {
        let mut world = setup_world();

        // Create a dummy item entity as target
        let item_entity = world.spawn(GridPosition { x: 5, y: 5 }).id();

        // Spawn a pop arriving at the item to haul
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: item_entity,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Haul,
                },
                AtTarget, // Arrived!
            ))
            .id();

        // Run the system
        world.run_system_once(arrival_handler_system).unwrap();

        // Assert AtTarget is still present (this will FAIL currently)
        assert!(
            world.get::<AtTarget>(pop).is_some(),
            "AtTarget should be preserved for Haul action so haul_system can see it"
        );

        // Assert MovementTarget is still present
        assert!(
            world.get::<MovementTarget>(pop).is_some(),
            "MovementTarget should be preserved for Haul action"
        );
    }

    #[test]
    fn test_full_hauling_cycle() {
        let mut world = setup_world();
        let schedule = build_simulation_schedule();
        world.add_schedule(schedule);

        // 1. Setup World
        // Pop at (0,0)
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                PopAction {
                    current: ActionType::Haul, // Force hauling action
                    current_utility: 1.0,
                    ticks_committed: 10,
                },
                scale::layer1::utility_ai::UtilityWeights::default(),
                scale::layer1::needs::Needs::default(),
            ))
            .id();

        // Item at (2,0) - Wood
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Wood,
                    amount: 10.0,
                },
                GridPosition { x: 2, y: 0 },
            ))
            .id();

        // Stockpile at (4,0)
        let _stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(),
                GridPosition { x: 4, y: 0 },
            ))
            .id();

        // 2. Run simulation ticks

        // Tick 1: movement (no-op), arrival (no-op), haul_system (finds item, sets target (2,0))
        world.run_schedule(SimulationSchedule);
        let mt = world.get::<MovementTarget>(pop);
        assert!(mt.is_some(), "Pop should target item");
        assert_eq!(mt.unwrap().target_position, GridPosition { x: 2, y: 0 });

        // Tick 2: movement (moves to 1,0), arrival (no-op), haul (no-op)
        world.run_schedule(SimulationSchedule);
        assert_eq!(world.get::<GridPosition>(pop).unwrap().x, 1);

        // Tick 3: movement (moves to 2,0, sets AtTarget), arrival (preserves AtTarget), haul (picks up)
        world.run_schedule(SimulationSchedule);

        // Verify Pickup Complete
        assert_eq!(world.get::<GridPosition>(pop).unwrap().x, 2);
        assert!(world.get_entity(item).is_err(), "Item should be despawned");
        assert!(
            world
                .get::<scale::layer1::resources::Carrying>(pop)
                .is_some(),
            "Pop should be carrying"
        );
        assert!(
            world.get::<AtTarget>(pop).is_none(),
            "AtTarget removed after pickup"
        );
        assert!(
            world.get::<MovementTarget>(pop).is_none(),
            "MovementTarget removed after pickup"
        );

        // Tick 4: movement (no-op), arrival (no-op), haul (finds stockpile, sets target (4,0))
        world.run_schedule(SimulationSchedule);
        let mt = world.get::<MovementTarget>(pop);
        assert!(mt.is_some(), "Pop should target stockpile");
        assert_eq!(mt.unwrap().target_position, GridPosition { x: 4, y: 0 });

        // Tick 5: movement (moves to 3,0)
        world.run_schedule(SimulationSchedule);
        assert_eq!(world.get::<GridPosition>(pop).unwrap().x, 3);

        // Tick 6: movement (moves to 4,0, sets AtTarget), arrival (preserves), haul (drops off)
        world.run_schedule(SimulationSchedule);

        // Verify Drop off Complete
        assert_eq!(world.get::<GridPosition>(pop).unwrap().x, 4);
        assert!(
            world
                .get::<scale::layer1::resources::Carrying>(pop)
                .is_none(),
            "Pop empty"
        );
        let resources = world.resource::<ColonyResources>();
        // Default wood is 15.0 + 10.0 hauled = 25.0
        assert!(
            (resources.wood - 25.0).abs() < f32::EPSILON,
            "Wood added to resources (expected 25.0)"
        );
    }
}
