#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::execution::{arrival_handler_system, AtTarget, MovementTarget};
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use scale::layer1::stockpile::Stockpile;
    use scale::layer1::structural_integrity::StructureCollapsed;
    use scale::layer1::utility_ai::{ActionType, PopAction};
    use scale::layer1::GridPosition;
    use scale::shared::time::SimulationTime;
    use scale::simulation::build_simulation_schedule;

    #[allow(dead_code)]
    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();
        let terrain = scale::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![scale::layer1::terrain::TerrainType::Grass; 100],
        };
        world.insert_resource(scale::layer1::fertility::FertilityGrid::from_terrain(
            &terrain,
        ));
        world.insert_resource(terrain);
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(scale::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::seasons::SeasonState::default());
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world.insert_resource(scale::layer1::building::BuildingMap::default()); // Fixed: Added BuildingMap
        world.insert_resource(scale::layer1::crowding::CrowdingGrid::new(10, 10)); // Fixed: Added CrowdingGrid
        world.insert_resource(scale::shared::log::MessageLog::default());
        world.insert_resource(scale::layer1::chronicle::Chronicle::default());
        world.insert_resource(scale::layer1::chronicle::BuildingTracker::default());
        world.insert_resource(scale::layer1::erosion::ErosionGrid::new(10, 10));
        world.insert_resource(scale::layer1::water::WaterGrid::new(10, 10));
        world.insert_resource(scale::layer1::environment::light_pollution::SkyGlow::default());
        world.insert_resource(scale::layer1::tech::TechState::default());
        world.insert_resource(scale::layer1::beauty::BeautyGrid::new(10, 10));
        world.insert_resource(scale::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10));
        world.insert_resource(scale::layer1::acoustic::NoiseMap::new(10, 10));
        world.insert_resource(scale::layer1::lighting::LightMap::new(10, 10));
        world.insert_resource(scale::layer1::lighting::AmbientLight::default());
        world.insert_resource(scale::layer1::wind::WindGrid::new(10, 10));
        world.insert_resource(scale::layer1::wind::GlobalWind::default());
        world.insert_resource(scale::layer1::atmosphere::BaseGlobalWind::default());
        world.insert_resource(scale::layer1::atmosphere::AtmosphereGrid::new(10, 10));
        world.insert_resource(scale::layer1::atmosphere::AtmosphericTide::default());
        world.insert_resource(scale::layer1::pressure::PressureGrid::new(10, 10));
        world.insert_resource(scale::layer1::notifications::NotificationQueue::default());
        world.insert_resource(scale::layer1::trade::MerchantState::default());
        world.insert_resource(scale::layer1::vermin::VerminState::default());
        world.insert_resource(scale::layer1::edicts::ColonyPolicies::default());
        world.init_resource::<Events<scale::layer1::chronicle::AddChronicleEvent>>();
        world.init_resource::<Events<scale::layer1::social::AffinityChange>>();
        world.init_resource::<Events<scale::layer1::pop::PopDied>>();
        world.init_resource::<Events<StructureCollapsed>>();
        world.init_resource::<Events<scale::layer1::heirloom::RetrogradeEngineeringEvent>>();
        world.init_resource::<Events<scale::layer1::energy::GridOverloadEvent>>();
        world.init_resource::<Events<scale::layer1::environment::hazards::AmputationEvent>>();
        world.init_resource::<Events<scale::layer1::geology::GeologicalEvent>>();
        world.init_resource::<Events<scale::layer1::society::InvestigationEvent>>();
        world.init_resource::<Events<scale::layer1::society::SuppressSocietyEvent>>();
        world.init_resource::<Events<scale::layer1::medical::PatientTreated>>();
        world.init_resource::<Events<scale::layer1::social::FavorChange>>();
        world.init_resource::<Events<scale::layer1::eureka::EurekaEvent>>();
        world.init_resource::<Events<scale::layer2::events::DetectionEvent>>();
        world.init_resource::<Events<scale::layer2::events::ShipDestroyedEvent>>();
        world.init_resource::<Events<scale::layer2::events::LaunchEvent>>();
        world.init_resource::<Events<scale::layer1::items::UnequipEvent>>();
        world.insert_resource(scale::layer1::social_mimicry::Trend::default());
        world.insert_resource(scale::layer1::visitor::VisitorSource::default());
        world.insert_resource(scale::layer1::weather::WeatherState::default());
        world.insert_resource(scale::layer1::quirks::PlanetaryTraits::default());
        world.insert_resource(scale::layer1::factions::Factions::default());
        world.insert_resource(scale::layer1::taboo::TabooState::default());
        world.insert_resource(scale::layer1::map::ScreenShake::default());
        world.insert_resource(scale::layer1::inspector::InspectorSource::default());
        world.insert_resource(scale::layer1::prototyping::BuildingMastery::default());
        world.init_resource::<scale::layer1::social::old_guard::Demographics>();
        world.init_resource::<scale::layer1::civic_ideology::ActiveIdeology>();
        world.insert_resource(scale::layer1::prototyping::BuildingMastery::default());
        world.insert_resource(scale::layer1::graffiti::GraffitiMap::default());
        world.insert_resource(scale::layer1::geology::SeismicGrid::new(10, 10));
        world.insert_resource(scale::layer1::environment::seismic::VibrationGrid::new(10, 10));
        world.insert_resource(scale::layer1::ecology::EcologyConfig::default());
        world.insert_resource(scale::layer1::society::SecretSocieties::default());
        world.insert_resource(scale::layer1::society::Unrest::default());
        world.insert_resource(scale::layer1::tech_envy::TechEnvyConfig::default());
        world.insert_resource(scale::layer2::visibility::SystemVisibility::default());
        world.insert_resource(scale::layer2::system::ViewMode::default());
        world.insert_resource(scale::layer2::system::SystemMap);
        world.insert_resource(scale::layer1::law::predictive_policing::PredictionConfig {
            threshold: 0.8,
            enabled: true,
        });
        world.init_resource::<Events<scale::layer3::silence::HostileSpawnEvent>>();
        world.init_resource::<Events<scale::layer1::direct_link::UnpossessEvent>>();
        world.init_resource::<Events<scale::layer1::direct_link::PossessEntityEvent>>();
        world.init_resource::<Events<scale::layer1::unrest::DenounceEvent>>();
        world.init_resource::<Events<scale::layer1::hologram::HologramFailureEvent>>();
        world.init_resource::<Events<scale::layer1::skills::XpGainEvent>>();
        world.init_resource::<Events<scale::layer1::construction::great_works::GreatWorkCompletedEvent>>();
        world.init_resource::<Events<scale::layer1::BuildingCompletedEvent>>();
        world.init_resource::<Events<scale::layer1::BuildingRemovedEvent>>();
        world.init_resource::<Events<scale::layer1::PopBorn>>();
        world.init_resource::<Events<scale::layer1::memory_core::HarvestMemoryCoreEvent>>();
        world.init_resource::<Events<scale::layer1::memory_core::ImplantMemoryCoreEvent>>();
        world.init_resource::<Events<scale::layer1::spiteful_will::OverrideWillEvent>>();
        world.init_resource::<Events<scale::layer1::spiteful_will::InheritanceEvent>>();
        world.init_resource::<Events<scale::layer1::logistics::orbital_drop::OrbitalDropEvent>>();
        world.init_resource::<Events<scale::layer1::ancestral_graves::SacrilegeEvent>>();
        world.init_resource::<Events<scale::layer1::void_weed::MerchantArrivalEvent>>();
        world.init_resource::<Events<scale::layer1::void_weed::PirateRaidEvent>>();
        world.init_resource::<Events<scale::layer1::overview_effect::ObserveEvent>>();
        world.init_resource::<Events<scale::layer1::tech::neural_leech::NeuralHubDeathEvent>>();
        world.init_resource::<Events<scale::layer1::geology::tectonic::MegaQuakeEvent>>();
        world.init_resource::<Events<scale::layer1::resources::MiningEvent>>();
        world.init_resource::<Events<scale::layer1::environment::volatile::ExplosionEvent>>();
        world.insert_resource(scale::shared::input::Input::default());
        world.insert_resource(scale::ui::UiState::default());
        world.insert_resource(scale::layer1::trade::TradeMarket::default());
        world.insert_resource(scale::layer1::solar::SolarCycleState::default());
        world.insert_resource(scale::layer1::clutter::ClutterGrid::new(10, 10));
        world.insert_resource(scale::shared::input::InputContextStack::default());
        world.insert_resource(scale::layer1::tech::infinite_archive::Archive::default());
        world.insert_resource(scale::layer1::shadow_market::ShadowMarketCooldown::default());
        world.insert_resource(scale::layer1::temperature::TemperatureGrid::new(
            10, 10, 20.0,
        ));
        world.insert_resource(scale::layer1::hum::HumMap::new(10, 10));
        world.insert_resource(scale::layer1::geology::tectonic::TectonicStress::default());
        world.insert_resource(scale::layer3::silence::DetectionRisk::default());
        world.insert_resource(scale::layer2::thermal::ThermalSignature::default());
        world.insert_resource(scale::layer1::social::empty_room::ActiveSanctuaries::default());
        world.insert_resource(scale::layer1::trade::TradeMarket::default());
        world.insert_resource(scale::layer1::void_weed::SmugglingHeat::default());
        world.insert_resource(scale::layer1::void_weed::TradeNetwork::default());
        world.init_resource::<scale::layer1::olfactory::ScentMap>();
        world.insert_resource(scale::layer1::solar::SolarCycleState::default());
        #[cfg(feature = "nova")]
        world.insert_resource(scale::layer1::machine_consciousness::ConsciousnessConfig::default());
        #[cfg(feature = "nova")]
        world.init_resource::<scale::layer1::constellations::Sky>();
        #[cfg(feature = "nova")]
        world.insert_resource(scale::layer1::void_signals::SignalNetwork::default());
        #[cfg(feature = "nova")]
        world.insert_resource(scale::layer1::loci::LociMap::new(10, 10));
        world.insert_resource(scale::layer1::clutter::ClutterGrid::new(10, 10));
        world.insert_resource(scale::layer1::environment::light_pollution::SkyGlow::default());
        world.insert_resource(scale::layer1::ColonyStats::default());
        world.insert_resource(scale::layer1::atmosphere::CorrosiveAtmosphere::default());
        world.insert_resource(scale::layer1::environment::terraforming::PlanetaryAtmosphere::default());
        world.insert_resource(scale::layer1::atmosphere::DiffusionConfig::default());
        world.insert_resource(scale::layer1::void_stare::VoidGrid::new(10, 10));
        world.insert_resource(scale::layer1::festivals::FestivalState::default());

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
        let mut world = scale::setup::setup_world();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::genetics::GeneSplicingResultEvent>>();

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
        let mut world = scale::setup::setup_world();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::genetics::GeneSplicingResultEvent>>();

        world.init_resource::<bevy_ecs::event::Events<scale::layer2::exploration::void_whispers::FleetReturnedEvent>>();
        world
            .init_resource::<bevy_ecs::event::Events<scale::layer1::whispering_ore::MinedOreEvent>>(
            );
        world.init_resource::<scale::layer2::trade::blockade::ColonyDebt>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer2::trade::penal_contracts::PrisonerDiedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<
            scale::layer3::planet::black_market_terraforming::RogueTerraformEvent,
        >>();
        world.init_resource::<scale::layer2::syzygy::SyzygyCycle>();
        world.init_resource::<scale::layer2::syzygy::PlanetaryGravity>();
        world.init_resource::<scale::layer2::syzygy::TidalForce>();
        world
            .init_resource::<bevy_ecs::event::Events<scale::layer1::grafting::GraftBuildingEvent>>(
            );
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::market::quantum_famine::ExportDumpEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::market::quantum_famine::MarketPanicEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer2::navigation::stellar_weather::FleetDamagedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<
            scale::layer2::events_new::reverse_quarantine::RefugeeFleetEvent,
        >>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::diplomacy_reflection::EntityKilledEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::diplomacy_reflection::FloraPlantedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::diplomacy_reflection::TraitChangedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer2::moon_hermits::PopDesertedEvent>>();

        world.init_resource::<bevy_ecs::event::Events<scale::layer2::exploration::void_whispers::FleetReturnedEvent>>();
        world
            .init_resource::<bevy_ecs::event::Events<scale::layer1::whispering_ore::MinedOreEvent>>(
            );
        world.init_resource::<scale::layer2::trade::blockade::ColonyDebt>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer2::trade::penal_contracts::PrisonerDiedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<
            scale::layer3::planet::black_market_terraforming::RogueTerraformEvent,
        >>();
        world.init_resource::<scale::layer2::syzygy::SyzygyCycle>();
        world.init_resource::<scale::layer2::syzygy::PlanetaryGravity>();
        world.init_resource::<scale::layer2::syzygy::TidalForce>();
        world
            .init_resource::<bevy_ecs::event::Events<scale::layer1::grafting::GraftBuildingEvent>>(
            );
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::market::quantum_famine::ExportDumpEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::market::quantum_famine::MarketPanicEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer2::navigation::stellar_weather::FleetDamagedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<
            scale::layer2::events_new::reverse_quarantine::RefugeeFleetEvent,
        >>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::diplomacy_reflection::EntityKilledEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::diplomacy_reflection::FloraPlantedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer3::diplomacy_reflection::TraitChangedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer2::moon_hermits::PopDesertedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer2::cascade::failure::LogisticsStrainedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer2::cascade::failure::DefenseWeakenedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::logistics::mass_driver::LaunchEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::logistics::mass_driver::BombardmentEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer2::cartographers_curse::SellTelemetryEvent>>();
        world
            .init_resource::<bevy_ecs::event::Events<scale::layer2::phantom::SpawnGhostFleetEvent>>(
            );

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
        scale::simulation::run_simulation_tick(&mut world);
        let mt = world.get::<MovementTarget>(pop);
        assert!(mt.is_some(), "Pop should target item");
        assert_eq!(mt.unwrap().target_position, GridPosition { x: 2, y: 0 });

        // Tick 2: movement (moves to 1,0), arrival (no-op), haul (no-op)
        scale::simulation::run_simulation_tick(&mut world);
        assert_eq!(world.get::<GridPosition>(pop).unwrap().x, 1);

        // Tick 3: movement (moves to 2,0, sets AtTarget), arrival (preserves AtTarget), haul (picks up)
        scale::simulation::run_simulation_tick(&mut world);

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
        scale::simulation::run_simulation_tick(&mut world);
        let mt = world.get::<MovementTarget>(pop);
        assert!(mt.is_some(), "Pop should target stockpile");
        assert_eq!(mt.unwrap().target_position, GridPosition { x: 4, y: 0 });

        // Tick 5: movement (moves to 3,0)
        scale::simulation::run_simulation_tick(&mut world);
        assert_eq!(world.get::<GridPosition>(pop).unwrap().x, 3);

        // Tick 6: movement (moves to 4,0, sets AtTarget), arrival (preserves), haul (drops off)
        scale::simulation::run_simulation_tick(&mut world);

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
