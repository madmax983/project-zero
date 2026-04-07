//! Shared world setup used by all entry points (native, headless, WASM).

use bevy_ecs::prelude::*;
use rand::RngCore;

// Fix: Unconditional import of AddChronicleEvent because init_resource usage is unconditional below
use crate::layer1::chronicle::AddChronicleEvent;

// use crate::layer1::chronicle::AddChronicleEvent; // Removed conditional import
use crate::layer1::heirloom::RetrogradeEngineeringEvent;
use crate::layer1::pop::{PopBorn, PopDied};
use crate::layer1::social::AffinityChange;
use crate::layer1::{
    generate_terrain, initial_chronicle_event, initial_naming_system, spawn_ancient_structures,
    spawn_initial_anomalies, spawn_initial_pops, AmbientLight, AtmosphereGrid, BuildMode,
    BuildingTracker, CameraCurrent, CameraTarget, Chronicle, ColonyPolicies, ColonyResources,
    DesignationMode, GlobalHitStop, LightMap, NamedLocations, NotificationQueue, OccupiedTiles,
    PopBundle, ScreenShake, SeasonState, TechState, TerrainType, UtilityConfig, Viewport,
};
use crate::shared::colony::ColonyName;
use crate::shared::input::{Input, InputContextStack};
use crate::shared::log::MessageLog;
use crate::shared::narrative::NarrativeGenerator;
use crate::shared::selection::Selection;
use crate::shared::state::GameState;
use crate::shared::time::{SimulationTime, WallTime};
use crate::shared::world_history::generate_world_history;
use crate::ui::map::RenderCache;
use crate::ui::state::UiState;

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
    world.init_resource::<crate::layer1::bio_acoustic_miasma::MiasmaRecordedSecret>();
    world.init_resource::<crate::layer1::stress::TraumaTracker>();
    world.init_resource::<bevy_ecs::event::Events<crate::layer2::skyhooks::LaunchIntent>>();
    world.init_resource::<crate::layer2::cartographers_curse::MapTelemetry>();
    world.init_resource::<bevy_ecs::event::Events<crate::layer2::cartographers_curse::SellTelemetryEvent>>();
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

    // Track artifacts to spawn entities for them
    let mut artifact_positions = Vec::new();
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            if terrain.get(x, y) == Some(crate::layer1::nature::terrain::TerrainType::Artifact) {
                artifact_positions.push((x, y));
            }
        }
    }

    world.insert_resource(terrain);
    world.insert_resource(fertility);
    world.insert_resource(roof);

    for (x, y) in artifact_positions {
        world.spawn((
            crate::layer1::nature::terrain::TerrainType::Artifact,
            crate::layer1::map::GridPosition {
                x: x as i32,
                y: y as i32,
            },
            crate::layer1::artifacts::ArtifactAura {
                radius: 5.0,
                effect: crate::layer1::artifacts::AuraEffect::Insight, // Default to Insight for map gen
            },
            crate::layer1::artifacts::Artifact,
        ));
    }

    world.insert_resource(Viewport::default());
    world.insert_resource(CameraTarget::default());
    world.insert_resource(CameraCurrent::default());
    world.insert_resource(ScreenShake::default());
    world.insert_resource(GlobalHitStop::default());
    world.insert_resource(SimulationTime::default());
    world.insert_resource(WallTime::default());
    world.insert_resource(BuildMode::default());
    world.insert_resource(DesignationMode::default());
    world.insert_resource(OccupiedTiles::default());
    world.insert_resource(crate::layer1::crowding::CrowdingGrid::new(80, 50));
    world.insert_resource(ColonyResources::default());
    world.insert_resource(crate::layer1::purity::PurityMap::default());
    world.insert_resource(ColonyPolicies::default());
    world.init_resource::<crate::layer1::civic_ideology::ActiveIdeology>();
    world.insert_resource(crate::layer1::social_mimicry::Trend::default());
    world.insert_resource(crate::layer1::factions::Factions::default());
    world.insert_resource(InputContextStack::default());
    world.insert_resource(MessageLog::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(crate::ui::tech::TechUiState::default());
    world.insert_resource(crate::ui::shell::ShellConfig::default());
    world.insert_resource(UiState::default());
    world.init_resource::<Input>();
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
    world.insert_resource(crate::layer1::trade::TradeMarket::default());
    world.insert_resource(crate::layer1::beauty::BeautyGrid::new(80, 50));
    world.insert_resource(crate::layer1::erosion::ErosionGrid::new(80, 50));
    world.insert_resource(crate::layer1::water::WaterGrid::new(80, 50));
    world.insert_resource(crate::layer1::zone::ZoneGrid::new(80, 50));
    world.insert_resource(crate::layer1::acoustic::NoiseMap::new(80, 50));
    world.init_resource::<crate::layer1::olfactory::ScentMap>();
    world.insert_resource(crate::layer1::hum::HumMap::new(80, 50));
    world.insert_resource(crate::layer1::void_stare::VoidGrid::new(80, 50));
    world.insert_resource(crate::layer1::clutter::ClutterGrid::new(80, 50));
    world.insert_resource(crate::layer1::social::empty_room::ActiveSanctuaries::default());
    world.insert_resource(AtmosphereGrid::new(80, 50));
    world.insert_resource(crate::layer1::wind::WindGrid::new(80, 50));
    world.insert_resource(crate::layer1::wind::GlobalWind::default());
    world.insert_resource(crate::layer1::atmosphere::BaseGlobalWind::default());
    world.insert_resource(crate::layer1::atmosphere::AtmosphericTide::default());
    world.insert_resource(crate::layer1::atmosphere::DiffusionConfig::default());
    world.init_resource::<crate::layer1::atmosphere::CorrosiveAtmosphere>();
    world.insert_resource(crate::layer1::pressure::PressureGrid::new(80, 50));
    world.insert_resource(crate::layer1::temperature::TemperatureGrid::new(
        80, 50, 15.0,
    ));
    world.insert_resource(crate::layer1::radioactive::RadiationGrid::new(80, 50));
    world.insert_resource(LightMap::new(80, 50));
    world.insert_resource(crate::layer1::light_pollution::SkyGlow::default());
    world.insert_resource(AmbientLight::default());
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
    world.insert_resource(crate::layer1::PlanetaryTraits::default());
    world.insert_resource(crate::layer2::syzygy::PlanetaryGravity::default());
    world.insert_resource(crate::layer1::weather::WeatherState::default());
    world.insert_resource(crate::layer1::terraforming::PlanetaryAtmosphere::default());
    world.insert_resource(crate::layer1::solar::SolarCycleState::default());
    world.insert_resource(crate::layer1::medical::MedicalPolicy::default());
    world.insert_resource(crate::layer1::prototyping::BuildingMastery::default());
    world.insert_resource(crate::layer1::graffiti::GraffitiMap::default());
    world.insert_resource(crate::layer1::geology::SeismicGrid::new(80, 50));
    world.insert_resource(crate::layer1::seismic::VibrationGrid::new(80, 50));
    world.insert_resource(crate::layer1::ecology::EcologyConfig::default());
    world.insert_resource(crate::layer1::social::society::SecretSocieties::default());
    world.insert_resource(crate::layer1::unrest::Unrest::default());
    world.insert_resource(crate::layer1::tech_envy::TechEnvyConfig::default());
    world.insert_resource(crate::layer1::building::BuildingMap::default());
    world.insert_resource(crate::layer1::law::predictive_policing::PredictionConfig {
        threshold: 0.8,
        enabled: true,
    });
    #[cfg(feature = "nova")]
    world.init_resource::<crate::layer1::oral_tradition::OralTradition>();
    #[cfg(feature = "nova")]
    world.insert_resource(crate::layer1::loci::LociMap::new(80, 50));
    world.init_resource::<crate::layer1::social::old_guard::Demographics>();
    world.insert_resource(crate::layer2::system::ViewMode::default());
    world.insert_resource(crate::layer2::system::SystemMap);
    world.insert_resource(crate::layer2::visibility::SystemVisibility::default());

    // Initialize System Generation (Layer 2)
    let mut rng = rand::thread_rng();
    let seed = crate::shared::random::WorldSeed(rng.next_u64());
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
    world.init_resource::<Events<crate::layer1::radio_nostalgia::BroadcastReceivedEvent>>();
    world.init_resource::<Events<AffinityChange>>();
    // world.init_resource::<Events<crate::layer1::DeathEvent>>();
    world.init_resource::<Events<PopDied>>();
    world.init_resource::<Events<PopBorn>>();
    world.init_resource::<Events<crate::layer1::events::BuildingCompletedEvent>>();
    world.init_resource::<Events<crate::layer1::parasitic_architecture::BuildingConsumedEvent>>();
    world.init_resource::<Events<crate::layer1::events::BuildingRemovedEvent>>();
    world.init_resource::<Events<crate::layer1::structural_integrity::StructureCollapsed>>();
    world.init_resource::<Events<RetrogradeEngineeringEvent>>();
    world.init_resource::<Events<crate::layer1::energy::GridOverloadEvent>>();
    world.init_resource::<Events<crate::layer1::geology::GeologicalEvent>>();
    world.init_resource::<Events<crate::layer1::social::society::InvestigationEvent>>();
    world.init_resource::<Events<crate::layer1::social::society::SuppressSocietyEvent>>();
    world.init_resource::<Events<crate::layer1::geology::tectonic::MegaQuakeEvent>>();
    world.init_resource::<Events<crate::layer1::ancestral_graves::SacrilegeEvent>>();
    world.init_resource::<Events<crate::layer1::resources::MiningEvent>>();
    world.init_resource::<Events<crate::layer1::shipbreaking::SpawnCrashedShipEvent>>();
    world.init_resource::<Events<crate::layer1::shipbreaking::MineEvent>>();
    world.init_resource::<crate::layer1::geology::tectonic::TectonicStress>();
    world.init_resource::<crate::layer1::tech::infinite_archive::Archive>();
    world.init_resource::<crate::layer2::trade::blockade::ColonyDebt>();
    world.init_resource::<crate::layer2::thermal::ThermalSignature>();
    world.init_resource::<Events<crate::layer1::medical::PatientTreated>>();
    world.init_resource::<Events<crate::layer1::genetics::GeneSplicingEvent>>();
    world.init_resource::<Events<crate::layer1::hazards::AmputationEvent>>();
    world.init_resource::<Events<crate::layer1::social::FavorChange>>();
    world.init_resource::<Events<crate::layer1::eureka::EurekaEvent>>();
    world.init_resource::<Events<crate::layer1::items::UnequipEvent>>();
    world.init_resource::<Events<crate::layer1::construction::GreatWorkCompletedEvent>>();
    world.init_resource::<Events<crate::layer2::trade::blockade::TradeShipArrivalEvent>>();
    world.init_resource::<Events<crate::layer2::events::LaunchEvent>>();
    world.init_resource::<Events<crate::layer2::events::ShipDestroyedEvent>>();
    world.init_resource::<Events<crate::layer3::map::HyperlaneCollapseEvent>>();
    world.init_resource::<Events<crate::layer3::map::TradeRouteSeveredEvent>>();
    world.init_resource::<Events<crate::layer2::moon_hermits::PopDesertedEvent>>();
    world.init_resource::<Events<crate::layer1::social::gossip_economy::GossipEvent>>();
    world.init_resource::<Events<crate::layer1::social::gossip_economy::BrokerPurchaseEvent>>();
    world.init_resource::<crate::layer1::social::gossip_economy::IntelTokens>();
    world.init_resource::<Events<crate::layer1::direct_link::PossessEntityEvent>>();
    world.init_resource::<Events<crate::layer1::direct_link::UnpossessEvent>>();
    world.init_resource::<Events<crate::layer1::volatile::ExplosionEvent>>();
    world.init_resource::<Events<crate::layer1::unrest::DenounceEvent>>();
    world.init_resource::<Events<crate::layer1::skills::XpGainEvent>>();
    world.init_resource::<Events<crate::layer1::hologram::HologramFailureEvent>>();
    world.init_resource::<Events<crate::layer1::temporal_ghost_towns::TemporalStutterEvent>>();
    world.init_resource::<Events<crate::layer2::events::DetectionEvent>>();
    world.init_resource::<Events<crate::layer1::logistics::orbital_drop::OrbitalDropEvent>>();
    world.init_resource::<Events<crate::layer1::economy::remittances::MigrantArrivalEvent>>();
    world.init_resource::<crate::layer1::economy::remittances::RemittanceTracker>();
    world.init_resource::<Events<crate::layer1::drone::DroneDisconnectedEvent>>();
    world.init_resource::<Events<crate::layer1::overview_effect::ObserveEvent>>();
    world.init_resource::<Events<crate::layer1::genetics::crop_modification::CropMutationEvent>>();
    world.init_resource::<Events<crate::layer1::tech::neural_leech::NeuralHubDeathEvent>>();

    world.init_resource::<crate::layer1::festivals::FestivalState>();
    world.init_resource::<crate::layer1::shadow_market::ShadowMarketCooldown>();
    world.insert_resource(crate::layer1::taboo::TabooState::default());

    // Initialize GPU compute context (non-fatal if no GPU available)
    // Skip on WASM since pollster::block_on doesn't work in browser context
    // Skip if headless mode is requested
    #[cfg(all(not(target_arch = "wasm32"), not(test)))]
    if !config.headless {
        match pollster::block_on(crate::gpu::context::GpuContext::new()) {
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
    #[cfg(feature = "nova")]
    world.init_resource::<crate::layer1::void_signals::SignalNetwork>();
    #[cfg(feature = "nova")]
    world.init_resource::<crate::experimental::genetic_memory::ColonyGeneticMemory>();

    world.init_resource::<crate::layer1::black_market::ColonyStats>();
    world.init_resource::<crate::layer1::void_weed::TradeNetwork>();
    world.init_resource::<crate::layer1::void_weed::SmugglingHeat>();

    world.init_resource::<Events<crate::layer1::void_weed::MerchantArrivalEvent>>();
    world.init_resource::<Events<crate::layer1::void_weed::PirateRaidEvent>>();

    world.init_resource::<crate::layer1::unrest::Unrest>();

    world.init_resource::<Events<crate::layer3::events::debt_prison::BailoutOfferEvent>>();
    world.init_resource::<Events<crate::layer3::events::debt_prison::AcceptBailoutEvent>>();

    world.init_resource::<Events<crate::layer1::memory_core::ImplantMemoryCoreEvent>>();
    world.init_resource::<Events<crate::layer1::memory_core::HarvestMemoryCoreEvent>>();
    world.init_resource::<Events<crate::layer1::tech::machine_awakening::BotGlitchEvent>>();
    world.init_resource::<Events<crate::layer1::orbital_tether::TetherSnapEvent>>();
    world
        .init_resource::<Events<crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent>>(
        );

    world.init_resource::<bevy::prelude::Time>();
    world.init_resource::<crate::layer1::tech::machine_awakening::GlobalSentience>();
    #[cfg(feature = "nova")]
    world.init_resource::<crate::layer1::machine_consciousness::ConsciousnessConfig>();
    world.init_resource::<crate::layer1::mind::fugue::FugueEventTracker>();

    world.insert_resource(ColonyName { name: colony_name });
    generate_world_history(&mut world);

    let starter_colony = spawn_starter_colony(&mut world);

    if let Some(layout) = starter_colony.as_ref() {
        spawn_initial_pops_at_positions(&mut world, &layout.pop_positions);
    } else {
        spawn_initial_pops(&mut world);
    }
    spawn_initial_anomalies(&mut world, 5);
    spawn_ancient_structures(
        &mut world,
        starter_colony.as_ref().map(|layout| layout.center),
        STARTER_HAZARD_BUFFER_RADIUS,
    );
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

const STARTER_WALL_OFFSETS: &[(i32, i32)] = &[
    (-2, -2),
    (-1, -2),
    (0, -2),
    (1, -2),
    (2, -2),
    (-2, -1),
    (2, -1),
    (-2, 0),
    (2, 0),
    (-2, 1),
    (2, 1),
    (-2, 2),
    (-1, 2),
    (1, 2),
    (2, 2),
];

const STARTER_AIRLOCK_OFFSET: (i32, i32) = (0, 2);
const STARTER_LANDER_OFFSET: (i32, i32) = (1, 0);
const STARTER_LIFE_SUPPORT_OFFSET: (i32, i32) = (-1, 0);
const STARTER_POP_OFFSETS: &[(i32, i32)] = &[(0, 0), (-1, -1), (0, -1), (1, -1), (0, 1)];
const STARTER_HAZARD_BUFFER_RADIUS: i32 = 14;

struct StarterColonyLayout {
    center: crate::layer1::GridPosition,
    pop_positions: Vec<crate::layer1::GridPosition>,
}

fn spawn_initial_pops_at_positions(world: &mut World, positions: &[crate::layer1::GridPosition]) {
    let mut rng = rand::thread_rng();
    for pos in positions {
        world.spawn(PopBundle::random(pos.x, pos.y, &mut rng));
    }
}

fn spawn_starter_colony(world: &mut World) -> Option<StarterColonyLayout> {
    let center = find_starter_colony_site(world)?;
    let interior_tiles: Vec<crate::layer1::GridPosition> = (-1..=1)
        .flat_map(|dy| {
            (-1..=1).map(move |dx| crate::layer1::GridPosition {
                x: center.x + dx,
                y: center.y + dy,
            })
        })
        .collect();

    for &(dx, dy) in STARTER_WALL_OFFSETS {
        let x = center.x + dx;
        let y = center.y + dy;
        crate::layer1::building::spawn_building_with_material(
            world,
            x,
            y,
            crate::layer1::building::BuildingType::Wall,
            crate::layer1::building::MaterialType::Metal,
        );
        world.resource_mut::<OccupiedTiles>().0.insert((x, y));
    }

    let airlock_pos = crate::layer1::GridPosition {
        x: center.x + STARTER_AIRLOCK_OFFSET.0,
        y: center.y + STARTER_AIRLOCK_OFFSET.1,
    };
    crate::layer1::building::spawn_building_with_material(
        world,
        airlock_pos.x,
        airlock_pos.y,
        crate::layer1::building::BuildingType::Airlock,
        crate::layer1::building::MaterialType::Metal,
    );
    world
        .resource_mut::<OccupiedTiles>()
        .0
        .insert((airlock_pos.x, airlock_pos.y));

    let lander_pos = crate::layer1::GridPosition {
        x: center.x + STARTER_LANDER_OFFSET.0,
        y: center.y + STARTER_LANDER_OFFSET.1,
    };
    crate::layer1::building::spawn_building_with_material(
        world,
        lander_pos.x,
        lander_pos.y,
        crate::layer1::building::BuildingType::Lander,
        crate::layer1::building::MaterialType::default(),
    );
    world
        .resource_mut::<OccupiedTiles>()
        .0
        .insert((lander_pos.x, lander_pos.y));

    let life_support_pos = crate::layer1::GridPosition {
        x: center.x + STARTER_LIFE_SUPPORT_OFFSET.0,
        y: center.y + STARTER_LIFE_SUPPORT_OFFSET.1,
    };
    crate::layer1::building::spawn_building_with_material(
        world,
        life_support_pos.x,
        life_support_pos.y,
        crate::layer1::building::BuildingType::LifeSupport,
        crate::layer1::building::MaterialType::default(),
    );
    world
        .resource_mut::<OccupiedTiles>()
        .0
        .insert((life_support_pos.x, life_support_pos.y));

    if let Some(mut pressure) = world.get_resource_mut::<crate::layer1::pressure::PressureGrid>() {
        for tile in &interior_tiles {
            pressure.set(tile.x, tile.y, 1.0);
        }
    }

    Some(StarterColonyLayout {
        center,
        pop_positions: STARTER_POP_OFFSETS
            .iter()
            .map(|(dx, dy)| crate::layer1::GridPosition {
                x: center.x + dx,
                y: center.y + dy,
            })
            .collect(),
    })
}

fn find_starter_colony_site(world: &World) -> Option<crate::layer1::GridPosition> {
    let (width, height) = {
        let terrain = world.resource::<crate::layer1::TerrainGrid>();
        (terrain.width, terrain.height)
    };
    let center_x = i32::try_from(width / 2).ok()?;
    let center_y = i32::try_from(height / 2).ok()?;
    let max_radius = center_x.max(center_y);

    for radius in 0..=max_radius {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if radius > 0 && dx.abs() != radius && dy.abs() != radius {
                    continue;
                }

                let candidate = crate::layer1::GridPosition {
                    x: center_x + dx,
                    y: center_y + dy,
                };

                if starter_colony_site_is_valid(world, candidate) {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

fn starter_colony_site_is_valid(world: &World, center: crate::layer1::GridPosition) -> bool {
    let terrain = world.resource::<crate::layer1::TerrainGrid>();
    let occupied = world.resource::<OccupiedTiles>();

    for dy in -2..=2 {
        for dx in -2..=2 {
            let x = center.x + dx;
            let y = center.y + dy;
            let Ok(ux) = usize::try_from(x) else {
                return false;
            };
            let Ok(uy) = usize::try_from(y) else {
                return false;
            };

            let Some(tile) = terrain.get(ux, uy) else {
                return false;
            };

            if !tile.is_walkable() {
                return false;
            }

            if occupied.0.contains(&(x, y)) {
                return false;
            }
        }
    }

    for &(dx, dy) in STARTER_WALL_OFFSETS {
        if !crate::layer1::building::can_place_building(world, center.x + dx, center.y + dy) {
            return false;
        }
    }

    for &(dx, dy) in &[
        STARTER_AIRLOCK_OFFSET,
        STARTER_LANDER_OFFSET,
        STARTER_LIFE_SUPPORT_OFFSET,
    ] {
        if !crate::layer1::building::can_place_building(world, center.x + dx, center.y + dy) {
            return false;
        }
    }

    true
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
