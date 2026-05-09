//! Shared simulation tick logic used by all entry points.
//!
//! Uses a Bevy `Schedule` to run all simulation systems. This enables:
//! - Typed system params (Bevy injects `Query`, `Res`, `ResMut` automatically)
//! - Automatic parallel execution of non-conflicting systems (with `multi_threaded`)
//! - `par_iter_mut` for intra-system parallelism on queries

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{IntoSystemConfigs, Schedule, ScheduleLabel};

use crate::gpu::evaluate::gpu_evaluate_actions;
use crate::layer1::building::{update_building_map_system, BuildingMap};
use crate::layer1::systems::{register_layer1_systems, update_event_buffer, Layer1SystemSet};
use crate::layer1::update_action_timer_system;
use crate::layer2::events::{DetectionEvent, LaunchEvent, ShipDestroyedEvent};
use crate::layer3::silence::{
    check_hostile_spawn_system, update_detection_risk_system, DetectionRisk, HostileSpawnEvent,
};
use crate::shared::time::SimulationTime;

/// Schedule label for the main simulation tick.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationSchedule;

/// Build the simulation schedule with all systems and ordering constraints.
///
/// Systems are organized into ordered groups matching the original sequential execution:
///
/// ```text
/// 1. AI Decision:     gpu_evaluate_actions → update_action_timer
/// 2. Execution:       cleanup_previous → process_start_plan → movement → arrival → work/haul
/// 3. Economy:         update_resource_caps, advance_season, produce_food, process_refining,
///                     process_research, restore_rest, restore_leisure (can run in parallel)
/// 4. Consumption:     consume_food → decay_needs → kill_starving → clean_dead_*
/// 5. Observation:     track_plan_outcomes, biography, dreams, milestones (can run in parallel)
/// 6. Tick increment:  (handled outside schedule)
/// ```
#[must_use]
pub fn build_simulation_schedule() -> Schedule {
    let mut schedule = Schedule::new(SimulationSchedule);
    register_simulation_core_systems(&mut schedule);
    register_simulation_extended_systems(&mut schedule);
    schedule
}

/// Run one simulation tick: all game systems via schedule, then increment tick counter.
#[allow(clippy::too_many_lines)]
fn init_simulation_resources(world: &mut World) {
    world.init_resource::<Events<crate::layer1::culture::nostalgia::RumorSpreadEvent>>();
    world.init_resource::<crate::layer1::biology::cybernetic_ascendancy::ColonyAverageUtility>();
    // Initialize schedule on first call (stored in World's Schedules resource)
    if !world.contains_resource::<Schedules>() {
        world.insert_resource(Schedules::default());
    }

    world.init_resource::<crate::layer1::social::old_guard::Demographics>();

    world.init_resource::<crate::layer1::diplomacy::factions::rivals::TerritoryGrid>();

    world.init_resource::<BuildingMap>();

    world.init_resource::<crate::layer1::stress::TraumaTracker>();
    world.init_resource::<Events<crate::layer2::skyhooks::LaunchIntent>>();
    world.init_resource::<Events<crate::layer1::tech::rogue_automation_cults::MachineCultFormedEvent>>();
    world.init_resource::<Events<crate::layer1::heirloom_tool::EquipHeirloomEvent>>();
    world.init_resource::<Events<crate::layer1::infrastructure::ancient::ConduitSurgeEvent>>();

    world.init_resource::<crate::layer1::tech_envy::TechEnvyConfig>();

    world.init_resource::<crate::layer1::shadow_market::ShadowMarketCooldown>();

    // Initialize Layer 2 Events
    world.init_resource::<Events<crate::layer1::geography::HistoricalEvent>>();
    world
        .init_resource::<Events<crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent>>(
        );
    world.init_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>();
    world.init_resource::<crate::layer1::diplomacy::factions::rivals::TerritoryGrid>();
    world.init_resource::<Events<crate::layer3::diplomacy::succession::SuccessionEvent>>();
    world.init_resource::<Events<crate::layer3::diplomacy::succession::SuccessionCrisisEvent>>();
    world.init_resource::<crate::layer1::mind::fugue::FugueEventTracker>();
    world.init_resource::<Events<crate::layer1::pop_memories::FamineEvent>>();
    world.init_resource::<Events<crate::layer1::diplomacy::wards::WarDeclaredEvent>>();
    if !world.contains_resource::<crate::layer1::diplomacy::wards::DiplomaticStanding>() {
        world.insert_resource(crate::layer1::diplomacy::wards::DiplomaticStanding {
            faction_relations: std::collections::HashMap::new(),
        });
    }

    world.init_resource::<Events<LaunchEvent>>();
    world.init_resource::<Events<ShipDestroyedEvent>>();
    world.init_resource::<Events<DetectionEvent>>();
    world.init_resource::<Events<HostileSpawnEvent>>();
    world.init_resource::<DetectionRisk>();
    world.init_resource::<Events<crate::layer1::unrest::DenounceEvent>>();
    world.init_resource::<Events<crate::layer1::law::embassy::ArrestEvent>>();
    world.init_resource::<Events<crate::layer1::law::embassy::DiplomaticIncidentEvent>>();
    world.init_resource::<Events<crate::layer1::environment::volatile::ExplosionEvent>>();

    if !world
        .contains_resource::<Events<crate::layer1::administration::edicts::TogglePolicyEvent>>()
    {
        world.init_resource::<Events<crate::layer1::administration::edicts::TogglePolicyEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::AccessDeniedEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::HackCentralHubEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::RevokePolicyEvent>>();
    }
    world.init_resource::<Events<crate::layer1::tectonic::MegaQuakeEvent>>();
    world.init_resource::<Events<crate::layer1::whispering_ore::MinedOreEvent>>();
    world.init_resource::<Events<crate::layer1::whispering_ore::MineSealedEvent>>();
    world.init_resource::<Events<crate::layer1::social::grievances::PostGrievanceEvent>>();
    world.init_resource::<Events<crate::layer1::deep_crust_resonance::ExcavationEvent>>();
    world.init_resource::<Events<crate::layer1::resources::MiningEvent>>();
    world.init_resource::<Events<crate::layer1::spiteful_will::InheritanceEvent>>();
    if !world
        .contains_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>()
    {
        world
            .init_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>();
    }
    world.init_resource::<crate::layer1::nature::biosphere_empathy::GlobalFloraHealth>();
    world.init_resource::<Events<crate::layer1::spiteful_will::OverrideWillEvent>>();
    world.init_resource::<crate::layer1::tectonic::TectonicStress>();

    world.init_resource::<crate::layer1::unrest::Unrest>();
    world.init_resource::<crate::layer1::atmosphere::CorrosiveAtmosphere>();

    // Initialize Thermal Bloom Resource
    world.init_resource::<crate::layer2::thermal::ThermalSignature>();

    // Initialize Detection Risk
    world.init_resource::<DetectionRisk>();

    world.init_resource::<crate::layer2::phantom::EmpireAutomationState>();
    world.init_resource::<Events<crate::layer2::trade::blockade::TradeShipArrivalEvent>>();
    world.init_resource::<Events<crate::layer2::trade::routes::SentientTollDemandEvent>>();
    if !world.contains_resource::<Events<crate::layer2::trade::escape_velocity::LaunchShipEvent>>()
    {
        world.init_resource::<Events<crate::layer2::trade::escape_velocity::LaunchShipEvent>>();
    }
    world.init_resource::<Events<crate::layer3::events::debt_prison::BailoutOfferEvent>>();
    if !world.contains_resource::<Events<crate::layer3::events::debt_prison::AcceptBailoutEvent>>()
    {
        world.init_resource::<Events<crate::layer3::events::debt_prison::AcceptBailoutEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer3::events::refugee_waves::RefugeeWaveEvent>>()
    {
        world.init_resource::<Events<crate::layer3::events::refugee_waves::RefugeeWaveEvent>>();
    }
    world.init_resource::<crate::layer2::trade::blockade::ColonyDebt>();

    world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>();
    world.init_resource::<Events<crate::layer2::silent_mutiny::SensorGlitchEvent>>();

    if !world
        .contains_resource::<Events<crate::layer1::nanite_fabrication::ContainmentBreachEvent>>()
    {
        world.init_resource::<Events<crate::layer1::nanite_fabrication::ContainmentBreachEvent>>();
    }

    world.init_resource::<Events<crate::layer1::social::sub_lithic::SabotageEvent>>();

    if !world
        .contains_resource::<Events<crate::layer2::trade::penal_contracts::PrisonerDiedEvent>>()
    {
        world.init_resource::<Events<crate::layer1::genetics::GeneSplicingEvent>>();
        world.init_resource::<Events<crate::layer1::genetics::GeneSplicingResultEvent>>();
        world.init_resource::<Events<crate::layer2::trade::penal_contracts::PrisonerDiedEvent>>();
    }

    world.init_resource::<Events<crate::layer2::governance::RebellionEvent>>();

    world.init_resource::<Events<crate::layer1::environment::disasters::DisasterEvent>>();

    world.init_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>();

    if !world.contains_resource::<Events<crate::layer1::agony_extract::HarvestAgonyExtractEvent>>()
    {
        world.init_resource::<Events<crate::layer1::agony_extract::HarvestAgonyExtractEvent>>();
        world.init_resource::<crate::layer1::agony_extract::AgonyExtractConfig>();
    }

    world.init_resource::<Events<crate::layer2::moon_hermits::PopDesertedEvent>>();
    world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();
    world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();

    world.init_resource::<Events<crate::layer1::environment::disasters::DisasterEvent>>();

    world.init_resource::<Events<crate::layer2::trade::biomass_tariff::TradeDeal>>();
    world.init_resource::<Events<crate::layer1::geodetic::GolemFormedEvent>>();

    world.init_resource::<crate::layer3::market::GalacticMarket>();

    world
        .init_resource::<Events<crate::layer2::events_new::reverse_quarantine::RefugeeFleetEvent>>(
        );
    world.init_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>();
    world.init_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>();
    world.init_resource::<crate::layer1::diplomacy::factions::rivals::TerritoryGrid>();
    world.init_resource::<Events<crate::layer3::planet::black_market_terraforming::RogueTerraformEvent>>();
    world.init_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>();
    world.init_resource::<Events<crate::layer3::market::quantum_famine::ExportDumpEvent>>();
    world.init_resource::<Events<crate::layer2::exploration::void_whispers::FleetReturnedEvent>>();
    world.init_resource::<Events<crate::layer1::core::integration::PirateAmnestyEvent>>();
    world.init_resource::<Events<crate::layer1::economy::resources::ResourceMinedEvent>>();
    world.init_resource::<crate::layer3::pirates::PirateThreatLevel>();
    world.init_resource::<crate::layer3::pirates::ResourceCurseSettings>();
    world.init_resource::<Events<crate::layer1::economy::resources::ResourceMinedEvent>>();
    world.init_resource::<crate::layer3::pirates::PirateThreatLevel>();
    world.init_resource::<crate::layer3::pirates::ResourceCurseSettings>();
    if !world
        .contains_resource::<Events<crate::layer2::navigation::stellar_weather::FleetDamagedEvent>>(
        )
    {
        world
            .init_resource::<Events<crate::layer2::navigation::stellar_weather::FleetDamagedEvent>>(
            );
    }

    world.init_resource::<Events<crate::layer1::environment::ignition::SparkEvent>>();
    world.init_resource::<Events<crate::layer1::environment::ignition::ExplosionEvent>>();

    world.init_resource::<Events<crate::layer1::environment::events::DebrisFallEvent>>();
    world.init_resource::<Events<crate::layer1::law::penal::OrganHarvestedEvent>>();
    world.init_resource::<crate::layer1::law::penal::ColonyInventory>();
    world.init_resource::<crate::layer3::council::GalacticCouncil>();
    world.init_resource::<crate::layer2::syzygy::SyzygyCycle>();
    world.init_resource::<crate::layer2::syzygy::PlanetaryGravity>();
    world.init_resource::<crate::layer2::syzygy::TidalForce>();

    // Digital Detritus
    world.init_resource::<crate::layer3::digital_detritus::DataMiningQueue>();
    world.init_resource::<crate::layer3::digital_detritus::DiscoveredTechs>();
    world.init_resource::<Events<crate::layer3::digital_detritus::VirusEvent>>();
    world.init_resource::<Events<crate::layer1::architecture::living_architecture::PopConsumedEvent>>();
    world.init_resource::<Events<crate::layer1::architecture::embezzlement::EmbezzlementEvent>>();
    world.init_resource::<crate::layer3::digital_detritus::JunkDataFilter>();

    world.init_resource::<Events<crate::layer1::logistics::mass_driver::LaunchEvent>>();
    world.init_resource::<Events<crate::layer1::logistics::mass_driver::BombardmentEvent>>();
    world.init_resource::<Events<crate::layer2::bombardment::BombardmentEvent>>();

    world.init_resource::<crate::layer3::council::GalacticCouncil>();

    world.init_resource::<crate::layer2::syzygy::SyzygyCycle>();
    world.init_resource::<crate::layer2::syzygy::PlanetaryGravity>();
    world.init_resource::<crate::layer2::syzygy::TidalForce>();
    // Initialize Infinite Archive Resource (Spec 248)
    world.init_resource::<crate::layer1::tech::infinite_archive::Archive>();

    world.init_resource::<Events<HostileSpawnEvent>>();

    world.init_resource::<crate::layer3::map::MapData>();
    world.init_resource::<Events<crate::layer3::map::FleetArrivalEvent>>();
    world.init_resource::<Events<crate::layer3::map::AnomalyDiscoveredEvent>>();
    world.init_resource::<Events<crate::layer3::diplomacy_reflection::EntityKilledEvent>>();
    world.init_resource::<Events<crate::layer3::diplomacy_reflection::FloraPlantedEvent>>();
    world.init_resource::<Events<crate::layer3::diplomacy_reflection::TraitChangedEvent>>();
    world.init_resource::<Events<crate::layer2::cascade::LogisticsStrainedEvent>>();
    world.init_resource::<crate::layer3::physics::relativity::SimulationTime>();
    world.init_resource::<Events<crate::layer2::cascade::DefenseWeakenedEvent>>();
    // Add our schedule if not yet added
    {
        let schedules = world.resource::<Schedules>();
        if schedules.get(SimulationSchedule).is_none() {
            world.init_resource::<Events<crate::layer3::ghost_ships::EvaluateTransitEvent>>();
            world
                .init_resource::<Events<crate::layer3::ghost_ships::EvaluateLostShipReturnEvent>>();
            world.init_resource::<Events<crate::layer1::unseen_bureaucracy::PhantomShiftEvent>>();
            world.init_resource::<Events<crate::layer3::diplomacy::cultural_ransom::RaidEvent>>();
            world.init_resource::<Events<crate::layer3::diplomacy::cultural_ransom::DiplomaticNegotiationEvent>>();
            world.init_resource::<crate::layer3::linguistic_drift::LinguisticNetwork>();
            world.init_resource::<Events<crate::layer3::linguistic_drift::CulturalSyncEvent>>();
            world.init_resource::<Events<crate::layer3::linguistic_drift::TradeEvent>>();
            world.init_resource::<Events<crate::layer1::logistics::beanstalk::BeanstalkEvent>>();
            world.init_resource::<Events<crate::layer3::treaty_cruisers::InspectionEvent>>();
            world.init_resource::<crate::layer3::treaty_cruisers::ActiveTreaties>();

            let mut schedule = build_simulation_schedule();
            schedule.add_systems((
                crate::layer1::logistics::beanstalk::beanstalk_morale_system,
                crate::layer1::logistics::beanstalk::beanstalk_collapse_system,
            ));
            world.add_schedule(schedule);
        }
    }
}

pub fn run_simulation_tick(world: &mut World) {
    init_simulation_resources(world);
    world.run_schedule(SimulationSchedule);
    world.resource_mut::<SimulationTime>().tick += 1;
    world
        .resource_mut::<crate::layer3::physics::relativity::SimulationTime>()
        .tick += 1;
}

fn register_simulation_core_systems(schedule: &mut Schedule) {
    // --- Register Core Layer 1 Systems ---
    register_layer1_systems(schedule);

    // Black Market Terraforming

    schedule.add_systems(
        (
            crate::layer1::deep_crust_resonance::resonant_ore_exposure_system,
            crate::layer1::deep_crust_resonance::resonance_social_spread_system,
            crate::layer1::core::integration::deep_crust_resonance_chronicle_bridge,
        )
            .in_set(Layer1SystemSet::Economy),
    );

    schedule.add_systems((
        crate::layer1::law::embassy::evaluate_diplomatic_crime_system,
        crate::layer1::law::embassy::process_diplomatic_arrest_system,
        crate::layer1::diplomacy::factions::rivals::rival_colony_expansion_system,
        crate::layer1::diplomacy::factions::rivals::rival_resource_drain_system,
    ));

    schedule.add_systems(crate::layer1::physics::harpoon::process_harpoon_impact_system);
    schedule.add_systems((crate::layer2::weather::weather_movement_system,));
    schedule.add_systems((
        crate::layer1::economy::apply_cultural_contraband_system,
        crate::layer3::planet::black_market_terraforming::trigger_rogue_terraforming,
        crate::layer3::planet::black_market_terraforming::apply_rogue_terraforming_events,
        crate::layer3::integration::black_market_terraforming_bridge,
        crate::layer2::station::process_megastructure_upkeep,
        crate::layer2::station::decommission_megastructure_system,
        crate::layer2::station::log_generous_gift_system,
    ));

    // Whispering Ore
    schedule.add_systems((
        crate::layer1::psionics::latent_awakening_system,
        crate::layer1::psionics::pyrokinesis_power_activation_system,
        crate::layer1::whispering_ore::process_whispering_ore_system,
        crate::layer1::whispering_ore::handle_mine_sealing_system,
    ));
}

#[allow(clippy::too_many_lines)]
fn register_simulation_extended_systems(schedule: &mut Schedule) {
    // --- Spec 622 ---
    schedule.add_systems((
        crate::layer1::biology::cybernetic_ascendancy::cybernetic_integration_system,
        crate::layer1::biology::cybernetic_ascendancy::update_colony_average_utility_system,
        crate::layer1::biology::cybernetic_ascendancy::cybernetic_mind_merge_system.after(
            crate::layer1::biology::cybernetic_ascendancy::update_colony_average_utility_system,
        ),
    ));
    // --- AI Decision Chain (GPU compute) ---
    schedule.add_systems((
        update_building_map_system,
        gpu_evaluate_actions.after(update_building_map_system),
        crate::layer1::visitor::visitor_behavior_system,
        crate::layer1::drone::evaluate_drone_actions_system.after(update_building_map_system),
        crate::layer1::social::secret_societies::secret_society_formation_system,
        crate::layer1::social::secret_societies::society_action_system,
        update_action_timer_system
            .after(gpu_evaluate_actions)
            .before(Layer1SystemSet::Execution),
    ));

    // --- Layer 3 Integration ---
    schedule.add_systems((
        (
            crate::layer2::cartographers_curse::process_telemetry_sale,
            crate::layer2::integration::cartographers_curse_chronicle_bridge,
            crate::layer2::integration::orbital_bombardment_chronicle_bridge,
            crate::layer2::integration::orbital_mirror_chronicle_bridge,
        )
            .chain(),
        crate::layer2::cartographers_curse::apply_drop_pod_accuracy,
        update_detection_risk_system.after(Layer1SystemSet::Economy),
        check_hostile_spawn_system.after(update_detection_risk_system),
        crate::layer3::market::ephemeral_market::spawn_ephemeral_market_system,
        crate::layer3::market::ephemeral_market::process_market_despawn_system,
        crate::layer3::market::ephemeral_market::fulfill_market_trade_system,
        crate::layer3::council::enforce_resolutions_system,
        crate::layer3::ghost_ships::evaluate_transit_system,
        crate::layer3::ghost_ships::evaluate_lost_ship_return_system,
        crate::layer3::treaty_cruisers::compliance_check_system,
    ));

    // --- Layer 2 Integration ---
    schedule.add_systems((
        crate::layer2::orbital_necropolis::apply_necropolis_bonus,
        crate::layer2::orbital_necropolis::handle_necropolis_destruction,
    ));
    schedule.add_systems((
        // Cleanup events
        update_event_buffer::<crate::layer1::administration::edicts::TogglePolicyEvent>,
        update_event_buffer::<crate::layer1::administration::edicts::AccessDeniedEvent>,
        update_event_buffer::<crate::layer1::administration::edicts::HackCentralHubEvent>,
        update_event_buffer::<crate::layer1::administration::edicts::RevokePolicyEvent>,
    ));
    schedule.add_systems((
        // Cleanup Layer 2 events
        update_event_buffer::<LaunchEvent>,
        update_event_buffer::<ShipDestroyedEvent>,
        update_event_buffer::<crate::layer2::orbital_necropolis::EntityDestroyedEvent>,
    ));
    schedule.add_systems((update_event_buffer::<DetectionEvent>,));
    schedule.add_systems((
        crate::layer2::integration::predecessor_orbital_shield_bridge_system,
        crate::layer2::fleet::fleet_order_system
            .after(crate::layer2::integration::predecessor_orbital_shield_bridge_system),
        crate::layer2::station::build_station_system,
        crate::layer2::station::zero_g_fermentation_system
            .after(crate::layer2::fleet::fleet_order_system)
            .after(crate::layer2::fleet::fleet_order_system),
        crate::layer2::derelict_stations::claim_station_system
            .after(crate::layer2::fleet::fleet_order_system),
        crate::layer2::fleet::fleet_movement_system.after(crate::layer2::fleet::fleet_order_system),
        crate::layer2::mutiny::decay_fleet_morale,
        crate::layer2::mutiny::evaluate_fleet_mutiny,
        crate::layer2::integration::primitive_retaliation_chronicle_bridge,
        crate::layer2::integration::assign_sensors_to_player_fleets_system
            .after(crate::layer2::fleet::fleet_movement_system),
        crate::layer2::sensor_ambiguity::resolve_sensors_system
            .after(crate::layer2::integration::assign_sensors_to_player_fleets_system),
        crate::layer2::integration::ensure_player_fleets_identified_system
            .after(crate::layer2::sensor_ambiguity::resolve_sensors_system),
        crate::layer1::integration::fleet_unload_system
            .after(crate::layer2::fleet::fleet_movement_system),
        crate::layer2::fleet::ensure_fleet_health_system,
        crate::layer2::combat::fleet_combat_system
            .after(crate::layer2::fleet::fleet_movement_system),
        crate::layer2::barnacles::barnacle_accumulation_system,
        // Debris Systems
        crate::layer2::orbit::kessler_gambit::trigger_kessler_gambit_system,
        crate::layer2::debris::debris_accumulation_system
            .after(crate::layer2::combat::fleet_combat_system),
        crate::layer2::debris::debris_attrition_system
            .after(crate::layer2::debris::debris_accumulation_system),
        crate::layer2::debris::debris_decay_system,
    ));
    schedule.add_systems((
        crate::layer2::orbit::debris_cult::evaluate_debris_cult_formation_system,
        crate::layer2::orbit::debris_cult::apply_debris_cult_morale_system
            .after(crate::layer2::debris::debris_attrition_system),
    ));

    schedule.add_systems((
        // Thermal Bloom Systems
        crate::layer2::thermal::update_thermal_bloom_system.after(Layer1SystemSet::Economy),
        crate::layer2::thermal::detection_risk_system
            .after(crate::layer2::thermal::update_thermal_bloom_system),
        crate::layer2::orbital_mirrors::orbital_mirror_focus_system,
        crate::layer2::integration::thermal_detection_handler_system
            .after(crate::layer2::thermal::detection_risk_system),
        crate::layer2::integration::founder_effect_bridge_system,
        crate::layer2::integration::escape_velocity_traits_bridge_system
            .before(crate::layer2::trade::escape_velocity::process_launch_system),
    ));

    schedule.add_systems((
        crate::layer2::cascade::evaluate_system_logistics,
        crate::layer2::cascade::update_sector_defenses
            .after(crate::layer2::cascade::evaluate_system_logistics),
        crate::layer2::cascade::calculate_invasion_threat
            .after(crate::layer2::cascade::update_sector_defenses),
        crate::layer2::integration::logistics_strained_chronicle_bridge
            .after(crate::layer2::cascade::evaluate_system_logistics),
        crate::layer2::integration::defense_weakened_chronicle_bridge
            .after(crate::layer2::cascade::update_sector_defenses),
    ));

    schedule.add_systems((
        crate::layer2::syzygy::update_syzygy_cycle_system,
        crate::layer2::syzygy::apply_syzygy_effects_system
            .after(crate::layer2::syzygy::update_syzygy_cycle_system),
        crate::layer2::integration::astrological_beliefs_bridge_system
            .after(crate::layer2::syzygy::update_syzygy_cycle_system),
        crate::layer2::trade::escape_velocity::process_launch_system
            .after(crate::layer2::syzygy::apply_syzygy_effects_system),
        crate::layer2::skyhooks::process_skyhook_launch
            .after(crate::layer2::syzygy::apply_syzygy_effects_system),
        crate::layer2::visibility::update_visibility_system.after(Layer1SystemSet::Economy),
        crate::layer2::visibility::enforce_view_mode_system
            .after(crate::layer2::visibility::update_visibility_system),
    ));

    schedule.add_systems((
        crate::layer2::orphan_fleet::hack_orphan_fleet_system,
        crate::layer2::orphan_fleet::orphan_fleet_defection_check_system,
        crate::layer2::orphan_fleet::process_orphan_defection_system
            .after(crate::layer2::orphan_fleet::orphan_fleet_defection_check_system),
    ));

    schedule.add_systems((
        crate::layer2::silent_mutiny::check_silent_mutiny_system,
        crate::layer2::silent_mutiny::process_mutiny_effects_system
            .after(crate::layer2::silent_mutiny::check_silent_mutiny_system),
        crate::layer2::integration::sensor_glitch_chronicle_bridge_system
            .after(crate::layer2::silent_mutiny::process_mutiny_effects_system),
        crate::layer2::integration::pre_trade_route_sync_system
            .before(crate::layer2::trade::routes::execute_trade_routes_system),
        crate::layer2::trade::routes::execute_trade_routes_system,
        crate::layer2::trade::routes::increase_route_complexity_system,
        crate::layer2::trade::routes::check_sentient_route_system,
        crate::layer2::integration::ideological_contraband_route_bridge
            .after(crate::layer2::trade::routes::execute_trade_routes_system),
        crate::layer2::integration::post_trade_route_sync_system
            .after(crate::layer2::trade::routes::execute_trade_routes_system),
    ));
    schedule.add_systems((
        crate::layer2::trade::biomass_tariff::process_biomass_tariff_system
            .after(crate::layer2::integration::post_trade_route_sync_system),
        crate::layer2::trade::penal_contracts::process_penal_contracts_system,
        crate::layer2::trade::penal_contracts::check_prisoner_status_system,
        crate::layer2::integration::penal_funds_to_resources_system,
        crate::layer2::integration::prisoner_death_chronicle_bridge_system,
        crate::layer2::trade::blockade::debt_blockade_system,
        crate::layer2::trade::blockade::blockade_interception_system
            .after(crate::layer2::trade::blockade::debt_blockade_system),
        crate::layer3::events::debt_prison::check_bailout_condition_system
            .after(crate::layer2::trade::blockade::blockade_interception_system),
        crate::layer3::events::debt_prison::process_bailout_acceptance_system
            .after(crate::layer3::events::debt_prison::check_bailout_condition_system),
        crate::layer3::market::update_market_prices_system,
        crate::layer3::diplomacy::diplomatic_negotiation_system,
        crate::layer3::diplomacy_reflection::aggregate_colony_stats,
        crate::layer3::diplomacy_reflection::update_diplomatic_traits
            .after(crate::layer3::diplomacy_reflection::aggregate_colony_stats),
        crate::layer3::diplomacy_reflection::apply_diplomatic_reactions
            .after(crate::layer3::diplomacy_reflection::update_diplomatic_traits),
    ));

    schedule.add_systems(
        (
            crate::layer3::diplomacy::succession::process_succession_system,
            crate::layer3::diplomacy::cultural_ransom::process_artifact_raid_system,
            crate::layer3::diplomacy::cultural_ransom::apply_hostage_penalties_system,
            crate::layer3::diplomacy::cultural_ransom::handle_ransom_negotiation_system,
            crate::layer3::diplomacy::red_tape_defense::process_bureaucracy_delays,
            crate::layer3::integration::dynastic_succession_chronicle_bridge,
            crate::layer3::integration::dynastic_crisis_chronicle_bridge,
        )
            .chain(),
    );

    schedule.add_systems((
        crate::layer3::map::trigger_hyperlane_collapse_system,
        crate::layer3::map::process_hyperlane_collapse_system
            .after(crate::layer3::map::trigger_hyperlane_collapse_system),
        crate::layer3::map::recalculate_trade_routes_system
            .after(crate::layer3::map::process_hyperlane_collapse_system),
        crate::layer3::integration::hyperlane_collapse_chronicle_bridge
            .after(crate::layer3::map::process_hyperlane_collapse_system),
    ));

    schedule.add_systems((crate::layer3::events::refugee_waves::process_refugee_decision,));

    schedule.add_systems((
        crate::layer2::phantom::check_scrapcode_threshold_system
            .after(crate::layer1::scrapcode::scrapcode_decay_system),
        crate::layer2::phantom::spawn_ghost_fleet_system
            .after(crate::layer2::phantom::check_scrapcode_threshold_system),
    ));

    schedule.add_systems((
        crate::layer2::governance::apply_governor_effects_system,
        crate::layer2::governance::update_governor_ambition_system
            .after(crate::layer2::governance::apply_governor_effects_system),
        crate::layer2::governance::check_governor_rebellion_system
            .after(crate::layer2::governance::update_governor_ambition_system),
        crate::layer2::integration::rebellion_chronicle_bridge_system
            .after(crate::layer2::governance::check_governor_rebellion_system),
        crate::layer2::tourism::process_disaster_tourism_system.after(Layer1SystemSet::Execution),
        crate::layer2::rogue_planets::rogue_planet_drift_system,
        crate::layer2::integration::process_grief_tourist_arrival_system
            .after(crate::layer2::tourism::process_disaster_tourism_system),
        crate::layer2::navigation::stellar_weather::apply_stellar_weather_effects,
        crate::layer2::integration::stellar_weather_damage_bridge_system
            .after(crate::layer2::navigation::stellar_weather::apply_stellar_weather_effects),
        crate::layer2::events_new::reverse_quarantine::process_refugee_decisions_system,
        crate::layer2::integration::reverse_quarantine_chronicle_bridge
            .after(crate::layer2::events_new::reverse_quarantine::process_refugee_decisions_system),
        crate::layer2::moon_hermits::process_hermit_desertions,
        crate::layer2::integration::moon_hermits_chronicle_bridge_system
            .after(crate::layer2::moon_hermits::process_hermit_desertions),
        crate::layer2::moon_hermits::hermit_theft_system,
        crate::layer2::empathic_plague::process_empathic_resonance
            .after(Layer1SystemSet::Observation),
        crate::layer3::pirates::evaluate_pirate_amnesty_system,
        crate::layer3::pirates::process_hyper_resources,
        crate::layer1::social::pirates::process_pirate_amnesty_system
            .after(crate::layer3::pirates::evaluate_pirate_amnesty_system),
        crate::layer1::social::pirates::pirate_crime_system,
    ));

    #[cfg(feature = "nova")]
    crate::experimental::gloom_sickness::register(schedule);
    #[cfg(feature = "nova")]
    crate::experimental::echo_chamber::register(schedule);
    #[cfg(feature = "nova")]
    crate::experimental::the_haunted_cartographer::register(schedule);
    #[cfg(feature = "nova")]
    crate::experimental::the_final_will::register(schedule);
    #[cfg(feature = "nova")]
    crate::experimental::digital_seance::register(schedule);
    #[cfg(feature = "nova")]
    crate::experimental::whispering_well::register(schedule);
    #[cfg(feature = "nova")]
    crate::experimental::echoing_footsteps::register(schedule);
    #[cfg(feature = "nova")]
    crate::experimental::panic_buying::register(schedule);
    #[cfg(feature = "nova")]
    crate::experimental::sleepwalking_hazards::register(schedule);
    #[cfg(feature = "nova")]
    crate::experimental::solar_flare_sickness::register(schedule);

    schedule.add_systems((
        crate::layer3::map::map_data_rot_system,
        crate::layer3::map::scout_ship_scan_system,
        crate::layer3::map::fleet_arrival_anomaly_system,
        crate::layer3::physics::relativity::process_time_dilation_system,
        crate::layer3::physics::relativity::update_fleet_local_time_system
            .after(crate::layer3::physics::relativity::process_time_dilation_system),
        crate::layer2::exploration::void_whispers::accumulate_void_whispers_in_deep_space,
        crate::layer2::exploration::void_whispers::spread_whispers_to_colony,
        crate::layer2::integration::void_whispers_chronicle_bridge,
    ));

    schedule.add_systems((crate::layer3::integration::jump_risk_bridge_system
        .after(crate::layer3::stellar_cartography::handle_jump_risk_system),));

    schedule.add_systems((
        crate::layer3::fleets::simulate_transit_drift_system,
        crate::layer3::fleets::apply_drift_on_foundation_system,
        // 1064 Digital Detritus
        crate::layer3::digital_detritus::process_data_mining_system,
        crate::layer3::digital_detritus::record_virus_event_chronicle_system,
    ));

    schedule.add_systems((
        crate::layer1::culture::gastronomers::spawn_gastronomer_faction_system,
        crate::layer1::culture::gastronomers::apply_culinary_singularity_buff_system,
        crate::layer1::core::integration::gastronomer_chronicle_bridge,
    ));
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::setup_world;
    use crate::shared::state::GameState;
    use bevy::prelude::Time;

    #[test]
    fn test_run_simulation_tick_increments() {
        let mut world = setup_world();
        world
            .init_resource::<crate::layer1::environment::bio_acoustic_miasma::MiasmaRecordedSecret>(
            );
        world.init_resource::<bevy::prelude::Events<crate::layer2::skyhooks::LaunchIntent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::infrastructure::ancient::ConduitSurgeEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::law::justice::CrimeCommittedEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::social::grievances::PostGrievanceEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::pop_memories::FamineEvent>>();
        world.init_resource::<crate::layer3::digital_detritus::DataMiningQueue>();
        world.init_resource::<crate::layer3::digital_detritus::DiscoveredTechs>();
        world.init_resource::<Events<crate::layer3::digital_detritus::VirusEvent>>();
        world.init_resource::<Events<crate::layer1::architecture::living_architecture::PopConsumedEvent>>();
        world
            .init_resource::<Events<crate::layer1::architecture::embezzlement::EmbezzlementEvent>>(
            );
        world.init_resource::<crate::layer3::digital_detritus::JunkDataFilter>();
        world.init_resource::<crate::layer1::culture::gastronomers::EmpireAdvancement>();
        world.init_resource::<bevy_ecs::event::Events<crate::layer1::culture::gastronomers::CulinarySingularityEvent>>();
        world.init_resource::<bevy_ecs::event::Events<crate::layer1::petrification::PopPetrifiedEvent>>();
        world.init_resource::<Events<crate::layer1::predecessors::WorldTriggerEvent>>();
        *world.resource_mut::<GameState>() = GameState::Running;

        let tick_before = world.resource::<SimulationTime>().tick;
        run_simulation_tick(&mut world);
        let tick_after = world.resource::<SimulationTime>().tick;

        assert_eq!(tick_after, tick_before + 1);
    }

    #[test]
    fn test_run_multiple_ticks() {
        let mut world = setup_world();
        world
            .init_resource::<crate::layer1::environment::bio_acoustic_miasma::MiasmaRecordedSecret>(
            );
        world.init_resource::<bevy::prelude::Events<crate::layer2::skyhooks::LaunchIntent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::law::justice::CrimeCommittedEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::social::grievances::PostGrievanceEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::pop_memories::FamineEvent>>();
        world.init_resource::<crate::layer3::digital_detritus::DataMiningQueue>();
        world.init_resource::<crate::layer3::digital_detritus::DiscoveredTechs>();
        world.init_resource::<Events<crate::layer3::digital_detritus::VirusEvent>>();
        world.init_resource::<Events<crate::layer1::architecture::living_architecture::PopConsumedEvent>>();
        world
            .init_resource::<Events<crate::layer1::architecture::embezzlement::EmbezzlementEvent>>(
            );
        world.init_resource::<crate::layer3::digital_detritus::JunkDataFilter>();
        *world.resource_mut::<GameState>() = GameState::Running;

        for _ in 0..10 {
            run_simulation_tick(&mut world);
        }

        assert_eq!(world.resource::<SimulationTime>().tick, 10);
    }

    #[test]
    fn test_schedule_builds_without_panic() {
        let _schedule = build_simulation_schedule();
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_schedule_runs_on_fresh_world() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        // Initialize Detection Risk for test
        world
            .init_resource::<crate::layer1::biology::cybernetic_ascendancy::ColonyAverageUtility>();
        world.init_resource::<crate::layer3::silence::DetectionRisk>();
        world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();
        world
            .init_resource::<crate::layer1::environment::bio_acoustic_miasma::MiasmaRecordedSecret>(
            );
        world.init_resource::<bevy::prelude::Events<crate::layer2::skyhooks::LaunchIntent>>();
        world.init_resource::<Events<crate::layer2::moon_hermits::PopDesertedEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::social::grievances::PostGrievanceEvent>>();
        world.init_resource::<Events<crate::layer1::law::embassy::ArrestEvent>>();
        world.init_resource::<Events<crate::layer1::law::embassy::DiplomaticIncidentEvent>>();
        world.init_resource::<Events<crate::layer3::silence::HostileSpawnEvent>>();
        world.init_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::TogglePolicyEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::AccessDeniedEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::HackCentralHubEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::RevokePolicyEvent>>();
        world.init_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>();
        world.init_resource::<crate::layer1::diplomacy::factions::rivals::TerritoryGrid>();
        world.init_resource::<Events<crate::layer1::temporal_ghost_towns::TemporalStutterEvent>>();
        world.init_resource::<Events<crate::layer3::planet::black_market_terraforming::RogueTerraformEvent>>();
        world.init_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>();
        world.init_resource::<Events<crate::layer3::market::quantum_famine::ExportDumpEvent>>();
        if !world
            .contains_resource::<Events<crate::layer3::diplomacy_reflection::EntityKilledEvent>>()
        {
            world.init_resource::<Events<crate::layer3::diplomacy_reflection::EntityKilledEvent>>();
        }
        if !world
            .contains_resource::<Events<crate::layer3::diplomacy_reflection::FloraPlantedEvent>>()
        {
            world.init_resource::<Events<crate::layer3::diplomacy_reflection::FloraPlantedEvent>>();
        }
        if !world
            .contains_resource::<Events<crate::layer3::diplomacy_reflection::TraitChangedEvent>>()
        {
            world.init_resource::<Events<crate::layer3::diplomacy_reflection::TraitChangedEvent>>();
        }
        world.init_resource::<crate::layer2::cartographers_curse::MapTelemetry>();
        world.init_resource::<Events<crate::layer2::cartographers_curse::SellTelemetryEvent>>();

        world.init_resource::<Events<crate::layer1::spiteful_will::InheritanceEvent>>();
        world.init_resource::<Events<crate::layer1::spiteful_will::OverrideWillEvent>>();
        world
            .init_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>();
        world.init_resource::<crate::layer2::phantom::EmpireAutomationState>();
        world.init_resource::<Events<crate::layer2::trade::blockade::TradeShipArrivalEvent>>();
        world.init_resource::<Events<crate::layer2::trade::routes::SentientTollDemandEvent>>();
        world.init_resource::<Events<crate::layer2::trade::escape_velocity::LaunchShipEvent>>();
        world.init_resource::<crate::layer2::trade::blockade::ColonyDebt>();

        world.init_resource::<Events<crate::layer3::events::debt_prison::BailoutOfferEvent>>();
        world.init_resource::<Events<crate::layer3::events::debt_prison::AcceptBailoutEvent>>();
        world.init_resource::<Events<crate::layer3::events::refugee_waves::RefugeeWaveEvent>>();
        world.init_resource::<crate::layer1::economy::smugglers_cove::ColonyAuthority>();

        world.init_resource::<Events<crate::layer1::diplomacy::wards::WarDeclaredEvent>>();
        if !world.contains_resource::<crate::layer1::diplomacy::wards::DiplomaticStanding>() {
            world.insert_resource(crate::layer1::diplomacy::wards::DiplomaticStanding {
                faction_relations: std::collections::HashMap::new(),
            });
        }

        world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>();

        world.init_resource::<Events<crate::layer2::silent_mutiny::SensorGlitchEvent>>();
        world.init_resource::<Events<crate::layer1::nanite_fabrication::ContainmentBreachEvent>>();
        world.init_resource::<Events<crate::layer1::social::sub_lithic::SabotageEvent>>();
        world.init_resource::<Events<crate::layer1::genetics::GeneSplicingEvent>>();
        world.init_resource::<Events<crate::layer1::genetics::GeneSplicingResultEvent>>();
        world.init_resource::<Events<crate::layer2::trade::penal_contracts::PrisonerDiedEvent>>();
        world.init_resource::<Events<crate::layer2::governance::RebellionEvent>>();
        world.init_resource::<Events<crate::layer1::environment::disasters::DisasterEvent>>();
        world.init_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>();
        world.init_resource::<Events<crate::layer1::agony_extract::HarvestAgonyExtractEvent>>();
        world.init_resource::<crate::layer1::agony_extract::AgonyExtractConfig>();
        world.init_resource::<Events<crate::layer2::trade::biomass_tariff::TradeDeal>>();
        world.init_resource::<Events<crate::layer1::geodetic::GolemFormedEvent>>();
        world.init_resource::<crate::layer3::council::GalacticCouncil>();
        world.init_resource::<crate::layer3::market::GalacticMarket>();
        world.init_resource::<crate::layer1::stress::TraumaTracker>();
        world.init_resource::<Events<crate::layer2::skyhooks::LaunchIntent>>();
        world.init_resource::<Events<crate::layer1::tech::rogue_automation_cults::MachineCultFormedEvent>>();
        world.init_resource::<Events<crate::layer1::heirloom_tool::EquipHeirloomEvent>>();
        world.init_resource::<Events<crate::layer2::events_new::reverse_quarantine::RefugeeFleetEvent>>();
        world.init_resource::<Events<crate::layer1::predecessors::WorldTriggerEvent>>();
        world.init_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>();
        world.init_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>();
        world.init_resource::<crate::layer1::diplomacy::factions::rivals::TerritoryGrid>();
        world.init_resource::<Events<crate::layer3::planet::black_market_terraforming::RogueTerraformEvent>>();
        world.init_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>();
        world.init_resource::<Events<crate::layer3::market::quantum_famine::ExportDumpEvent>>();
        world
            .init_resource::<Events<crate::layer2::exploration::void_whispers::FleetReturnedEvent>>(
            );
        world.init_resource::<Events<crate::layer1::core::integration::PirateAmnestyEvent>>();
        world.init_resource::<Events<crate::layer1::economy::resources::ResourceMinedEvent>>();
        world.init_resource::<crate::layer3::pirates::PirateThreatLevel>();
        world.init_resource::<crate::layer3::pirates::ResourceCurseSettings>();
        world.init_resource::<Events<crate::layer1::law::justice::CrimeCommittedEvent>>();
        world.init_resource::<Events<crate::layer1::pop_memories::FamineEvent>>();
        world
            .init_resource::<Events<crate::layer2::navigation::stellar_weather::FleetDamagedEvent>>(
            );
        world.init_resource::<Events<crate::layer1::environment::ignition::SparkEvent>>();
        world.init_resource::<Events<crate::layer1::environment::ignition::ExplosionEvent>>();
        world.init_resource::<Events<crate::layer1::environment::events::DebrisFallEvent>>();
        world.init_resource::<Events<crate::layer1::law::penal::OrganHarvestedEvent>>();
        world.init_resource::<crate::layer1::law::penal::ColonyInventory>();
        world.init_resource::<Events<crate::layer1::whispering_ore::MinedOreEvent>>();
        world.init_resource::<Events<crate::layer1::whispering_ore::MineSealedEvent>>();
        world.init_resource::<Events<crate::layer1::deep_crust_resonance::ExcavationEvent>>();
        world.init_resource::<crate::layer3::council::GalacticCouncil>();
        world.init_resource::<crate::layer2::syzygy::SyzygyCycle>();
        world.init_resource::<crate::layer2::syzygy::PlanetaryGravity>();
        world.init_resource::<crate::layer2::syzygy::TidalForce>();

        world.init_resource::<crate::layer3::map::MapData>();
        world.init_resource::<Events<crate::layer3::map::FleetArrivalEvent>>();
        world.init_resource::<Events<crate::layer3::map::AnomalyDiscoveredEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy_reflection::EntityKilledEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy_reflection::FloraPlantedEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy_reflection::TraitChangedEvent>>();

        world.init_resource::<Events<crate::layer1::logistics::mass_driver::LaunchEvent>>();
        world.init_resource::<Events<crate::layer1::logistics::mass_driver::BombardmentEvent>>();
        world.init_resource::<Events<crate::layer2::bombardment::BombardmentEvent>>();
        world.init_resource::<Events<crate::layer2::bombardment::BombardmentEvent>>();

        world.init_resource::<Events<crate::layer2::cascade::LogisticsStrainedEvent>>();
        world.init_resource::<Events<crate::layer2::cascade::DefenseWeakenedEvent>>();

        world.init_resource::<crate::layer3::physics::relativity::SimulationTime>();

        world.init_resource::<Events<crate::layer2::moon_hermits::PopDesertedEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::social::grievances::PostGrievanceEvent>>();

        world.init_resource::<Time>();
        world.init_resource::<Events<crate::layer2::primitives::InvasionEvent>>();
        world.init_resource::<Events<crate::layer2::orphan_fleet::HackOrphanFleetEvent>>();
        world.init_resource::<Events<crate::layer2::orphan_fleet::OrphanFleetDefectionEvent>>();

        world.init_resource::<Events<crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy::succession::SuccessionEvent>>();
        world.init_resource::<Events<crate::layer1::pop_memories::FamineEvent>>();
        world
            .init_resource::<Events<crate::layer3::diplomacy::succession::SuccessionCrisisEvent>>();

        world.init_resource::<Events<crate::layer3::ghost_ships::EvaluateTransitEvent>>();
        world.init_resource::<Events<crate::layer3::ghost_ships::EvaluateLostShipReturnEvent>>();
        world.init_resource::<Events<crate::layer1::unseen_bureaucracy::PhantomShiftEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy::cultural_ransom::RaidEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy::cultural_ransom::DiplomaticNegotiationEvent>>();
        world.init_resource::<crate::layer3::linguistic_drift::LinguisticNetwork>();
        world.init_resource::<Events<crate::layer3::linguistic_drift::CulturalSyncEvent>>();
        world.init_resource::<Events<crate::layer3::linguistic_drift::TradeEvent>>();
        world.init_resource::<Events<crate::layer1::logistics::beanstalk::BeanstalkEvent>>();
        world.init_resource::<Events<crate::layer3::treaty_cruisers::InspectionEvent>>();
        world.init_resource::<crate::layer3::treaty_cruisers::ActiveTreaties>();

        world.init_resource::<crate::layer3::digital_detritus::DataMiningQueue>();
        world.init_resource::<crate::layer3::digital_detritus::DiscoveredTechs>();
        world.init_resource::<Events<crate::layer3::digital_detritus::VirusEvent>>();
        world.init_resource::<Events<crate::layer1::architecture::living_architecture::PopConsumedEvent>>();
        world
            .init_resource::<Events<crate::layer1::architecture::embezzlement::EmbezzlementEvent>>(
            );
        world.init_resource::<crate::layer3::digital_detritus::JunkDataFilter>();
        world.init_resource::<crate::layer1::culture::gastronomers::EmpireAdvancement>();
        world.init_resource::<bevy_ecs::event::Events<crate::layer1::culture::gastronomers::CulinarySingularityEvent>>();
        world.init_resource::<bevy_ecs::event::Events<crate::layer1::petrification::PopPetrifiedEvent>>();
        world.init_resource::<Events<crate::layer1::predecessors::WorldTriggerEvent>>();

        let schedule = build_simulation_schedule();
        world.add_schedule(schedule);
        world.run_schedule(SimulationSchedule);

        // Should not panic — all systems run correctly on a fresh world
    }
}
