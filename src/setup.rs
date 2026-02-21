//! Shared world setup used by all entry points (native, headless, WASM).

use bevy_ecs::prelude::*;
use rand::RngCore;

#[cfg(all(not(target_arch = "wasm32"), not(test)))]
use crate::gpu::context::GpuContext;
use crate::layer1::chronicle::AddChronicleEvent;
use crate::layer1::heirloom::RetrogradeEngineeringEvent;
use crate::layer1::pop::PopDied;
use crate::layer1::social::AffinityChange;
use crate::layer1::{
    AmbientLight, AtmosphereGrid, BuildMode, BuildingTracker, CameraCurrent, CameraTarget,
    Chronicle, ChronicleUiState, ColonyPolicies, ColonyResources, DesignationMode, LightMap,
    NamedLocations, NotificationQueue, OccupiedTiles, ScreenShake, SeasonState, TechState,
    TerrainType, UtilityConfig, Viewport, generate_terrain, initial_chronicle_event,
    initial_naming_system, spawn_ancient_structures, spawn_initial_anomalies, spawn_initial_pops,
};
use crate::shared::colony::ColonyName;
use crate::shared::input::InputContextStack;
use crate::shared::log::MessageLog;
use crate::shared::narrative::NarrativeGenerator;
use crate::shared::selection::Selection;
use crate::shared::state::GameState;
use crate::shared::time::{SimulationTime, WallTime};
use crate::shared::world_history::generate_world_history;
use crate::ui::map::RenderCache;

/// Ensures the Bevy task pools are initialized (required for `par_iter_mut`).
///
/// Safe to call multiple times — uses `get_or_init` internally.
pub fn init_task_pools() {
    bevy_tasks::ComputeTaskPool::get_or_init(bevy_tasks::TaskPool::default);
}

/// Configuration for world setup.
#[derive(Debug, Default, Clone, Copy)]
pub struct SetupConfig {
    /// If true, skip GPU initialization (for headless environments).
    pub headless: bool,
}

/// Create and initialize a new game world with all resources.
#[must_use]
pub fn setup_world() -> World {
    setup_world_with_config(SetupConfig::default())
}

/// Create and initialize a new game world with custom configuration.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn setup_world_with_config(#[allow(unused_variables)] config: SetupConfig) -> World {
    init_task_pools();
    let mut world = World::new();
    world.insert_resource(GameState::default());
    world.insert_resource(MenuState::default());

    let terrain = generate_terrain(80, 50);
    let mut roof =
        crate::layer1::structural_integrity::RoofGrid::new(terrain.width, terrain.height);
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            if terrain.get(x, y) == Some(crate::layer1::TerrainType::Rock) {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                roof.set(x as i32, y as i32, true);
            }
        }
    }
    let fertility = crate::layer1::fertility::FertilityGrid::from_terrain(&terrain);
    world.insert_resource(terrain);
    world.insert_resource(fertility);
    world.insert_resource(roof);

    world.insert_resource(Viewport::default());
    world.insert_resource(CameraTarget::default());
    world.insert_resource(CameraCurrent::default());
    world.insert_resource(ScreenShake::default());
    world.insert_resource(SimulationTime::default());
    world.insert_resource(WallTime::default());
    world.insert_resource(BuildMode::default());
    world.insert_resource(DesignationMode::default());
    world.insert_resource(OccupiedTiles::default());
    world.insert_resource(crate::layer1::crowding::CrowdingGrid::new(80, 50));
    world.insert_resource(ColonyResources::default());
    world.insert_resource(crate::layer1::purity::PurityMap::default());
    world.insert_resource(ColonyPolicies::default());
    world.insert_resource(crate::layer1::social_mimicry::Trend::default());
    world.insert_resource(crate::layer1::factions::Factions::default());
    world.insert_resource(InputContextStack::default());
    world.insert_resource(MessageLog::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(ChronicleUiState::default());
    world.insert_resource(NotificationQueue::default());
    world.insert_resource(BuildingTracker::default());
    world.insert_resource(crate::layer1::vermin::VerminState::default());
    world.insert_resource(Selection::default());
    world.insert_resource(RenderCache::default());
    world.insert_resource(UtilityConfig::default());
    world.insert_resource(crate::layer1::sleepwalking::SleepwalkingConfig::default());
    world.insert_resource(SeasonState::default());
    world.insert_resource(NamedLocations::default());
    world.insert_resource(TechState::default());
    world.insert_resource(crate::layer1::trade::MerchantState::default());
    world.insert_resource(crate::layer1::beauty::BeautyGrid::new(80, 50));
    world.insert_resource(crate::layer1::erosion::ErosionGrid::new(80, 50));
    world.insert_resource(crate::layer1::water::WaterGrid::new(80, 50));
    world.insert_resource(crate::layer1::zone::ZoneGrid::new(80, 50));
    world.insert_resource(crate::layer1::acoustic::NoiseMap::new(80, 50));
    world.insert_resource(AtmosphereGrid::new(80, 50));
    world.insert_resource(crate::layer1::wind::WindGrid::new(80, 50));
    world.insert_resource(crate::layer1::wind::GlobalWind::default());
    world.insert_resource(crate::layer1::pressure::PressureGrid::new(80, 50));
    world.insert_resource(crate::layer1::temperature::TemperatureGrid::new(
        80, 50, 15.0,
    ));
    world.insert_resource(LightMap::new(80, 50));
    world.insert_resource(AmbientLight::default());
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
    world.insert_resource(crate::layer1::PlanetaryTraits::default());
    world.insert_resource(crate::layer1::weather::WeatherState::default());
    world.insert_resource(crate::layer1::medical::MedicalPolicy::default());
    world.insert_resource(crate::layer1::prototyping::BuildingMastery::default());
    world.insert_resource(crate::layer1::graffiti::GraffitiMap::default());
    world.insert_resource(crate::layer1::geology::SeismicGrid::new(80, 50));
    world.insert_resource(crate::layer1::seismic::VibrationGrid::new(80, 50));
    world.insert_resource(crate::layer1::ecology::EcologyConfig::default());
    world.insert_resource(crate::layer1::society::SecretSocieties::default());
    world.insert_resource(crate::layer1::society::Unrest::default());
    world.insert_resource(crate::layer1::tech_envy::TechEnvyConfig::default());
    world.insert_resource(crate::layer1::building::BuildingMap::default());
    world.insert_resource(crate::layer1::predictive_policing::PredictionConfig {
        threshold: 0.8,
        enabled: true,
    });
    #[cfg(feature = "nova")]
    world.init_resource::<crate::layer1::oral_tradition::OralTradition>();
    world.init_resource::<crate::layer1::social::old_guard::Demographics>();
    world.insert_resource(crate::layer2::system::ViewMode::default());
    world.insert_resource(crate::layer2::system::SystemMap);
    world.insert_resource(crate::layer2::visibility::SystemVisibility::default());

    // Initialize System Generation (Layer 2)
    let mut rng = rand::thread_rng();
    let seed = crate::layer2::generation::WorldSeed(rng.next_u64());
    world.insert_resource(seed);

    // Run generation system once
    let mut system_schedule = Schedule::default();
    system_schedule.add_systems(crate::layer2::generation::generate_system);
    system_schedule.run(&mut world);

    initialize_visitor_source(&mut world);
    world.insert_resource(crate::layer1::inspector::InspectorSource {
        next_visit_tick: 5000,
    });

    world.init_resource::<Events<AddChronicleEvent>>();
    world.init_resource::<Events<AffinityChange>>();
    world.init_resource::<Events<crate::layer1::DeathEvent>>();
    world.init_resource::<Events<PopDied>>();
    world.init_resource::<Events<crate::layer1::structural_integrity::StructureCollapsed>>();
    world.init_resource::<Events<RetrogradeEngineeringEvent>>();
    world.init_resource::<Events<crate::layer1::energy::GridOverloadEvent>>();
    world.init_resource::<Events<crate::layer1::geology::GeologicalEvent>>();
    world.init_resource::<Events<crate::layer1::society::InvestigationEvent>>();
    world.init_resource::<Events<crate::layer1::society::SuppressSocietyEvent>>();
    world.init_resource::<Events<crate::layer1::medical::PatientTreated>>();
    world.init_resource::<Events<crate::layer1::hazards::AmputationEvent>>();
    world.init_resource::<Events<crate::layer1::social::FavorChange>>();

    world.insert_resource(crate::layer1::taboo::TabooState::default());

    // Initialize GPU compute context (non-fatal if no GPU available)
    // Skip on WASM since pollster::block_on doesn't work in browser context
    // Skip if headless mode is requested
    #[cfg(all(not(target_arch = "wasm32"), not(test)))]
    if !config.headless {
        match pollster::block_on(GpuContext::new()) {
            Ok(ctx) => {
                world.insert_resource(ctx);
            }
            Err(e) => {
                eprintln!("GPU init failed ({e}), falling back to CPU evaluate");
            }
        }
    }

    let generator = NarrativeGenerator::from_embedded();
    let colony_name = generator.generate_star_name();
    world.insert_resource(generator);
    #[cfg(feature = "nova")]
    world.init_resource::<crate::layer1::constellations::Sky>();

    world.insert_resource(ColonyName { name: colony_name });
    generate_world_history(&mut world);

    spawn_initial_pops(&mut world);
    spawn_initial_anomalies(&mut world, 5);
    spawn_ancient_structures(&mut world);
    initial_naming_system(&mut world);
    initial_chronicle_event(&mut world);

    world
}

fn initialize_visitor_source(world: &mut World) {
    let terrain = world.resource::<crate::layer1::TerrainGrid>();
    let mut spawn_points = Vec::new();
    // Top and Bottom edges
    for x in 0..terrain.width {
        if terrain.get(x, 0).is_some_and(TerrainType::is_walkable) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            spawn_points.push(crate::layer1::GridPosition { x: x as i32, y: 0 });
        }
        if terrain
            .get(x, terrain.height - 1)
            .is_some_and(TerrainType::is_walkable)
        {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            spawn_points.push(crate::layer1::GridPosition {
                x: x as i32,
                y: (terrain.height - 1) as i32,
            });
        }
    }
    // Left and Right edges (excluding corners already added)
    for y in 1..(terrain.height - 1) {
        if terrain.get(0, y).is_some_and(TerrainType::is_walkable) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            spawn_points.push(crate::layer1::GridPosition { x: 0, y: y as i32 });
        }
        if terrain
            .get(terrain.width - 1, y)
            .is_some_and(TerrainType::is_walkable)
        {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            spawn_points.push(crate::layer1::GridPosition {
                x: (terrain.width - 1) as i32,
                y: y as i32,
            });
        }
    }

    world.insert_resource(crate::layer1::VisitorSource {
        spawn_points,
        next_spawn_tick: 500,
    });
}

use crate::shared::menu::MenuState;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::{Pop, TerrainGrid};
    use crate::shared::colony::ColonyName;
    use crate::shared::narrative::NarrativeGenerator;

    #[test]
    fn test_setup_world_creates_resources() {
        let world = setup_world();

        assert!(world.contains_resource::<GameState>());
        assert!(world.contains_resource::<SimulationTime>());
        assert!(world.contains_resource::<TerrainGrid>());
        assert!(world.contains_resource::<ColonyResources>());
        assert!(world.contains_resource::<Chronicle>());
        assert!(world.contains_resource::<Selection>());
        assert!(world.contains_resource::<NotificationQueue>());
        assert!(world.contains_resource::<RenderCache>());
        assert!(world.contains_resource::<UtilityConfig>());
        assert!(world.contains_resource::<SeasonState>());
        assert!(world.contains_resource::<NamedLocations>());
        assert!(world.contains_resource::<TechState>());
        assert!(world.contains_resource::<NarrativeGenerator>());
        assert!(world.contains_resource::<ColonyName>());
    }

    #[test]
    fn test_setup_world_spawns_pops() {
        let mut world = setup_world();
        let pop_count = world.query::<&Pop>().iter(&world).count();
        assert!(pop_count > 0, "Should have spawned initial pops");
    }

    #[test]
    fn test_setup_world_has_chronicle_event() {
        let world = setup_world();
        let chronicle = world.resource::<Chronicle>();
        assert!(
            !chronicle.events.is_empty(),
            "Should have initial chronicle event"
        );
    }

    #[test]
    fn test_setup_world_default_state_is_main_menu() {
        let world = setup_world();
        assert_eq!(*world.resource::<GameState>(), GameState::MainMenu);
    }

    #[test]
    fn test_setup_world_creates_factions() {
        let world = setup_world();
        assert!(world.contains_resource::<crate::layer1::factions::Factions>());
    }

    #[test]
    fn test_setup_world_generates_system() {
        use crate::layer2::generation::{Planet, Star};
        let mut world = setup_world();

        let stars = world.query::<&Star>().iter(&world).count();
        assert_eq!(stars, 1, "Should generate exactly one star");

        let planets = world.query::<&Planet>().iter(&world).count();
        assert!(planets >= 3, "Should generate at least 3 planets");
    }
}
