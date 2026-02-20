#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::GridPosition;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::drone::{Drone, DroneHub};
    use scale::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use scale::layer1::stockpile::Stockpile;
    use scale::layer1::utility_types::PopAction;
    use scale::shared::time::SimulationTime;
    use scale::simulation::{SimulationSchedule, build_simulation_schedule};

    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();

        // Essential Resources
        world.insert_resource(scale::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![scale::layer1::terrain::TerrainType::Grass; 100],
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(scale::layer1::utility_types::UtilityConfig::default());
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world.insert_resource(scale::layer1::building::BuildingMap::default());
        world.insert_resource(scale::shared::log::MessageLog::default());

        // Event Buffers
        world.init_resource::<Events<scale::layer1::chronicle::AddChronicleEvent>>();
        world.init_resource::<Events<scale::layer1::social::AffinityChange>>();
        world.init_resource::<Events<scale::layer1::DeathEvent>>();
        world.init_resource::<Events<scale::layer1::pop::PopDied>>();
        world.init_resource::<Events<scale::layer1::structural_integrity::StructureCollapsed>>();
        world.init_resource::<Events<scale::layer1::heirloom::RetrogradeEngineeringEvent>>();
        world.init_resource::<Events<scale::layer1::energy::GridOverloadEvent>>();
        world.init_resource::<Events<scale::layer1::hazards::AmputationEvent>>();
        world.init_resource::<Events<scale::layer1::geology::GeologicalEvent>>();
        world.init_resource::<Events<scale::layer1::society::InvestigationEvent>>();
        world.init_resource::<Events<scale::layer1::society::SuppressSocietyEvent>>();
        world.init_resource::<Events<scale::layer1::medical::PatientTreated>>();
        world.init_resource::<Events<scale::layer1::social::FavorChange>>();

        // Other dependencies for systems
        world.insert_resource(scale::layer1::crowding::CrowdingGrid::new(10, 10));
        world.insert_resource(scale::layer1::erosion::ErosionGrid::new(10, 10));
        world.insert_resource(scale::layer1::geology::SeismicGrid::new(10, 10));
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
        world.insert_resource(scale::layer1::vermin::VerminState::default());
        world.insert_resource(scale::layer1::edicts::ColonyPolicies::default());
        world.insert_resource(scale::layer1::social_mimicry::Trend::default());
        world.insert_resource(scale::layer1::visitor::VisitorSource::default());
        world.insert_resource(scale::layer1::weather::WeatherState::default());
        world.insert_resource(scale::layer1::quirks::PlanetaryTraits::default());
        world.insert_resource(scale::layer1::factions::Factions::default());
        world.insert_resource(scale::layer1::taboo::TabooState::default());
        world.insert_resource(scale::layer1::map::ScreenShake::default());
        world.insert_resource(scale::layer1::inspector::InspectorSource::default());
        world.init_resource::<scale::layer1::social::old_guard::Demographics>();
        world.insert_resource(scale::layer1::tech_envy::TechEnvyConfig::default());
        world.insert_resource(scale::shared::colony::ColonyName {
            name: "Test".to_string(),
        });
        let generator = scale::shared::narrative::NarrativeGenerator::from_embedded();
        world.insert_resource(generator);

        // Missing resources fix
        world.insert_resource(scale::layer1::chronicle::Chronicle::default());
        world.insert_resource(scale::layer1::chronicle::BuildingTracker::default());
        world.insert_resource(scale::layer1::prototyping::BuildingMastery::default());
        world.insert_resource(scale::layer1::seasons::SeasonState::default());
        world.insert_resource(scale::layer1::temperature::TemperatureGrid::new(
            10, 10, 20.0,
        ));
        world.insert_resource(scale::layer1::graffiti::GraffitiMap::default());
        world.insert_resource(scale::layer1::ecology::EcologyConfig::default());
        world.insert_resource(scale::layer1::society::SecretSocieties::default());
        world.insert_resource(scale::layer1::society::Unrest::default());
        world.insert_resource(scale::layer1::sleepwalking::SleepwalkingConfig::default());
        world.insert_resource(scale::layer1::medical::MedicalPolicy::default());
        world.insert_resource(scale::layer1::building::BuildMode::default());
        world.insert_resource(scale::layer1::designation::DesignationMode::default());
        world.insert_resource(scale::layer1::ChronicleUiState::default()); // Fixed
        world.insert_resource(scale::shared::selection::Selection::default());
        world.insert_resource(scale::shared::input::InputContextStack::default());
        world.insert_resource(scale::ui::map::RenderCache::default());
        world.insert_resource(scale::layer1::map::CameraCurrent::default());
        world.insert_resource(scale::layer1::map::CameraTarget::default());
        world.insert_resource(scale::layer1::Viewport::default()); // Fixed
        world.insert_resource(scale::shared::time::WallTime::default());
        world.insert_resource(scale::layer1::purity::PurityMap::default());
        world.insert_resource(scale::layer1::social::old_guard::Demographics::default());

        world.insert_resource(scale::layer2::system::ViewMode::default());
        world.insert_resource(scale::layer2::system::SystemMap);
        world.insert_resource(scale::layer2::visibility::SystemVisibility::default());

        // Blackout Protocol (Energy)
        world.insert_resource(scale::layer1::energy::BlackoutProtocol::default());

        world.insert_resource(Schedules::default());

        // Add schedule
        let schedule = build_simulation_schedule();
        world.add_schedule(schedule);

        world
    }

    #[test]
    fn test_drone_spawning() {
        let mut world = setup_world();

        // Place a DroneHub at (0,0)
        // Note: We use spawn_building helper logic by manually inserting components
        // or just spawn what DroneHub typically has.
        // DroneHub needs Power to be active? Usually spawners work if powered.
        // For MVP test, let's assume it has power or provide a source.

        // Spawn Power Source (Generator) at (0,1)
        world.spawn((
            Building {
                building_type: BuildingType::Generator,
            },
            GridPosition { x: 0, y: 1 },
            scale::layer1::energy::PowerSource {
                output: 100.0,
                active: true,
            },
            scale::layer1::energy::Conduit, // Connect to hub
        ));

        // Spawn DroneHub at (0,0)
        world.spawn((
            Building {
                building_type: BuildingType::DroneHub,
            },
            GridPosition { x: 0, y: 0 },
            DroneHub,
            scale::layer1::energy::PowerConsumer {
                demand: 10.0,
                active: false,
            },
            scale::layer1::energy::Conduit,
        ));

        // Run simulation for a few ticks to allow power grid to update and spawner to run
        for _ in 0..10 {
            world.run_schedule(SimulationSchedule);
        }

        // Check for Drones
        let drone_count = world.query::<&Drone>().iter(&world).count();
        assert!(drone_count > 0, "DroneHub should spawn drones when powered");
    }

    #[test]
    fn test_drone_hauling() {
        let mut world = setup_world();

        // 1. Setup Infrastructure
        // Power Source
        world.spawn((
            Building {
                building_type: BuildingType::Generator,
            },
            GridPosition { x: 0, y: 0 },
            scale::layer1::energy::PowerSource {
                output: 100.0,
                active: true,
            },
            scale::layer1::energy::Conduit,
        ));

        // DroneHub
        world.spawn((
            Building {
                building_type: BuildingType::DroneHub,
            },
            GridPosition { x: 1, y: 0 },
            DroneHub,
            scale::layer1::energy::PowerConsumer {
                demand: 10.0,
                active: false,
            },
            scale::layer1::energy::Conduit,
        ));

        // Stockpile at (9,0)
        world.spawn((
            Building {
                building_type: BuildingType::Stockpile,
            },
            GridPosition { x: 9, y: 0 },
            Stockpile::default(),
        ));

        // Resource Item at (5,0) - Wood
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Wood,
                    amount: 10.0,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        // 2. Run simulation
        // Wait for spawn
        let mut drone_spawned = false;
        for _ in 0..10 {
            world.run_schedule(SimulationSchedule);
            if world.query::<&Drone>().iter(&world).count() > 0 {
                drone_spawned = true;
                break;
            }
        }
        assert!(drone_spawned, "Drone should spawn");

        // Wait for haul (approx 20 ticks to travel back and forth)
        // 1 (Hub) -> 5 (Item) = 4 ticks
        // 5 (Item) -> 9 (Stockpile) = 4 ticks
        // Plus some decision delay. 20 ticks should be enough.
        for _ in 0..50 {
            world.run_schedule(SimulationSchedule);
            if world.get_entity(item).is_err() {
                // Item despawned (picked up)
            }
        }

        // Check if item is gone (picked up)
        assert!(world.get_entity(item).is_err(), "Item should be picked up");

        // Check resources (Wood should increase by 10)
        // Default wood is 15.0. Expected 25.0.
        let res = world.resource::<ColonyResources>();
        assert!(
            (res.wood - 25.0).abs() < f32::EPSILON,
            "Resources should be delivered. Current: {}",
            res.wood
        );
    }
}
