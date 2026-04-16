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
#[allow(clippy::too_many_lines)]
pub fn build_simulation_schedule() -> Schedule {
    let mut schedule = Schedule::new(SimulationSchedule);

    // --- Register Core Layer 1 Systems ---
    register_layer1_systems(&mut schedule);
    // Black Market Terraforming
    schedule.add_systems((crate::layer2::weather::weather_movement_system,));
    schedule.add_systems((
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

    // --- AI Decision Chain (GPU compute) ---
    schedule.add_systems((
        update_building_map_system,
        gpu_evaluate_actions.after(update_building_map_system),
        crate::layer1::visitor::visitor_behavior_system,
        crate::layer1::drone::evaluate_drone_actions_system.after(update_building_map_system),
        update_action_timer_system
            .after(gpu_evaluate_actions)
            .before(Layer1SystemSet::Execution),
    ));

    // --- Layer 3 Integration ---
    schedule.add_systems((
        crate::layer2::cartographers_curse::process_telemetry_sale,
        crate::layer2::cartographers_curse::apply_drop_pod_accuracy,
        update_detection_risk_system.after(Layer1SystemSet::Economy),
        check_hostile_spawn_system.after(update_detection_risk_system),
        crate::layer3::council::enforce_resolutions_system,
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
    schedule.add_systems((
        update_event_buffer::<DetectionEvent>,
        crate::layer2::fleet::fleet_order_system,
        crate::layer2::station::build_station_system,
        crate::layer2::station::zero_g_fermentation_system
            .after(crate::layer2::fleet::fleet_order_system)
            .after(crate::layer2::fleet::fleet_order_system),
        crate::layer2::derelict_stations::claim_station_system
            .after(crate::layer2::fleet::fleet_order_system),
        crate::layer2::fleet::fleet_movement_system.after(crate::layer2::fleet::fleet_order_system),
        crate::layer2::mutiny::decay_fleet_morale,
        crate::layer2::mutiny::evaluate_fleet_mutiny,
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
        crate::layer2::debris::debris_accumulation_system
            .after(crate::layer2::combat::fleet_combat_system),
        crate::layer2::debris::debris_attrition_system
            .after(crate::layer2::debris::debris_accumulation_system),
        crate::layer2::debris::debris_decay_system
            .after(crate::layer2::debris::debris_attrition_system),
    ));

    schedule.add_systems((
        // Thermal Bloom Systems
        crate::layer2::thermal::update_thermal_bloom_system.after(Layer1SystemSet::Economy),
        crate::layer2::thermal::detection_risk_system
            .after(crate::layer2::thermal::update_thermal_bloom_system),
        crate::layer2::integration::thermal_detection_handler_system
            .after(crate::layer2::thermal::detection_risk_system),
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
        crate::layer2::trade::escape_velocity::process_launch_system
            .after(crate::layer2::syzygy::apply_syzygy_effects_system),
        crate::layer2::skyhooks::process_skyhook_launch
            .after(crate::layer2::syzygy::apply_syzygy_effects_system),
        crate::layer2::visibility::update_visibility_system.after(Layer1SystemSet::Economy),
        crate::layer2::visibility::enforce_view_mode_system
            .after(crate::layer2::visibility::update_visibility_system),
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
        crate::layer2::integration::post_trade_route_sync_system
            .after(crate::layer2::trade::routes::execute_trade_routes_system),
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
        crate::layer1::social::pirates::process_pirate_amnesty_system
            .after(crate::layer3::pirates::evaluate_pirate_amnesty_system),
        crate::layer1::social::pirates::pirate_crime_system,
    ));

    #[cfg(feature = "nova")]
    crate::experimental::echo_chamber::register(&mut schedule);

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
    ));

    schedule
}

/// Run one simulation tick: all game systems via schedule, then increment tick counter.
pub fn run_simulation_tick(world: &mut World) {
    // Initialize schedule on first call (stored in World's Schedules resource)
    if !world.contains_resource::<Schedules>() {
        world.insert_resource(Schedules::default());
    }

    if !world.contains_resource::<crate::layer1::social::old_guard::Demographics>() {
        world.init_resource::<crate::layer1::social::old_guard::Demographics>();
    }

    if !world.contains_resource::<BuildingMap>() {
        world.init_resource::<BuildingMap>();
    }

    if !world.contains_resource::<crate::layer1::stress::TraumaTracker>() {
        world.init_resource::<crate::layer1::stress::TraumaTracker>();
        world.init_resource::<Events<crate::layer2::skyhooks::LaunchIntent>>();
    }

    if !world.contains_resource::<crate::layer1::tech_envy::TechEnvyConfig>() {
        world.init_resource::<crate::layer1::tech_envy::TechEnvyConfig>();
    }

    if !world.contains_resource::<crate::layer1::shadow_market::ShadowMarketCooldown>() {
        world.init_resource::<crate::layer1::shadow_market::ShadowMarketCooldown>();
    }

    // Initialize Layer 2 Events
    if !world.contains_resource::<Events<crate::layer1::geography::HistoricalEvent>>() {
        world.init_resource::<Events<crate::layer1::geography::HistoricalEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent>>() {
        world.init_resource::<Events<crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>() {
        world.init_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer3::diplomacy::succession::SuccessionEvent>>() {
        world.init_resource::<Events<crate::layer3::diplomacy::succession::SuccessionEvent>>();
        world
            .init_resource::<Events<crate::layer3::diplomacy::succession::SuccessionCrisisEvent>>();
        world.init_resource::<crate::layer1::mind::fugue::FugueEventTracker>();
    }
    if !world.contains_resource::<Events<crate::layer1::pop_memories::FamineEvent>>() {
        world.init_resource::<Events<crate::layer1::pop_memories::FamineEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::diplomacy::wards::WarDeclaredEvent>>() {
        world.init_resource::<Events<crate::layer1::diplomacy::wards::WarDeclaredEvent>>();
    }
    if !world.contains_resource::<crate::layer1::diplomacy::wards::DiplomaticStanding>() {
        world.insert_resource(crate::layer1::diplomacy::wards::DiplomaticStanding {
            faction_relations: std::collections::HashMap::new(),
        });
    }

    if !world.contains_resource::<Events<LaunchEvent>>() {
        world.init_resource::<Events<LaunchEvent>>();
    }
    if !world.contains_resource::<Events<ShipDestroyedEvent>>() {
        world.init_resource::<Events<ShipDestroyedEvent>>();
    }
    if !world.contains_resource::<Events<DetectionEvent>>() {
        world.init_resource::<Events<DetectionEvent>>();
    }
    if !world.contains_resource::<Events<HostileSpawnEvent>>() {
        world.init_resource::<Events<HostileSpawnEvent>>();
    }
    if !world.contains_resource::<DetectionRisk>() {
        world.init_resource::<DetectionRisk>();
    }
    if !world.contains_resource::<Events<crate::layer1::unrest::DenounceEvent>>() {
        world.init_resource::<Events<crate::layer1::unrest::DenounceEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::environment::volatile::ExplosionEvent>>() {
        world.init_resource::<Events<crate::layer1::environment::volatile::ExplosionEvent>>();
    }

    if !world
        .contains_resource::<Events<crate::layer1::administration::edicts::TogglePolicyEvent>>()
    {
        world.init_resource::<Events<crate::layer1::administration::edicts::TogglePolicyEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::AccessDeniedEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::HackCentralHubEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::RevokePolicyEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::geology::tectonic::MegaQuakeEvent>>() {
        world.init_resource::<Events<crate::layer1::geology::tectonic::MegaQuakeEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::whispering_ore::MinedOreEvent>>() {
        world.init_resource::<Events<crate::layer1::whispering_ore::MinedOreEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::whispering_ore::MineSealedEvent>>() {
        world.init_resource::<Events<crate::layer1::whispering_ore::MineSealedEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::resources::MiningEvent>>() {
        world.init_resource::<Events<crate::layer1::resources::MiningEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::spiteful_will::InheritanceEvent>>() {
        world.init_resource::<Events<crate::layer1::spiteful_will::InheritanceEvent>>();
    }
    if !world
        .contains_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>()
    {
        world
            .init_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>();
    }
    if !world.contains_resource::<crate::layer1::nature::biosphere_empathy::GlobalFloraHealth>() {
        world.init_resource::<crate::layer1::nature::biosphere_empathy::GlobalFloraHealth>();
    }
    if !world.contains_resource::<Events<crate::layer1::spiteful_will::OverrideWillEvent>>() {
        world.init_resource::<Events<crate::layer1::spiteful_will::OverrideWillEvent>>();
    }
    if !world.contains_resource::<crate::layer1::geology::tectonic::TectonicStress>() {
        world.init_resource::<crate::layer1::geology::tectonic::TectonicStress>();
    }

    if !world.contains_resource::<crate::layer1::unrest::Unrest>() {
        world.init_resource::<crate::layer1::unrest::Unrest>();
    }
    if !world.contains_resource::<crate::layer1::atmosphere::CorrosiveAtmosphere>() {
        world.init_resource::<crate::layer1::atmosphere::CorrosiveAtmosphere>();
    }

    // Initialize Thermal Bloom Resource
    if !world.contains_resource::<crate::layer2::thermal::ThermalSignature>() {
        world.init_resource::<crate::layer2::thermal::ThermalSignature>();
    }

    // Initialize Detection Risk
    if !world.contains_resource::<DetectionRisk>() {
        world.init_resource::<DetectionRisk>();
    }

    if !world.contains_resource::<crate::layer2::phantom::EmpireAutomationState>() {
        world.init_resource::<crate::layer2::phantom::EmpireAutomationState>();
    }
    if !world.contains_resource::<Events<crate::layer2::trade::blockade::TradeShipArrivalEvent>>() {
        world.init_resource::<Events<crate::layer2::trade::blockade::TradeShipArrivalEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer2::trade::escape_velocity::LaunchShipEvent>>()
    {
        world.init_resource::<Events<crate::layer2::trade::escape_velocity::LaunchShipEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer3::events::debt_prison::BailoutOfferEvent>>() {
        world.init_resource::<Events<crate::layer3::events::debt_prison::BailoutOfferEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer3::events::debt_prison::AcceptBailoutEvent>>()
    {
        world.init_resource::<Events<crate::layer3::events::debt_prison::AcceptBailoutEvent>>();
    }
    if !world.contains_resource::<crate::layer2::trade::blockade::ColonyDebt>() {
        world.init_resource::<crate::layer2::trade::blockade::ColonyDebt>();
    }

    if !world.contains_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>() {
        world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer2::silent_mutiny::SensorGlitchEvent>>() {
        world.init_resource::<Events<crate::layer2::silent_mutiny::SensorGlitchEvent>>();
    }

    if !world
        .contains_resource::<Events<crate::layer1::nanite_fabrication::ContainmentBreachEvent>>()
    {
        world.init_resource::<Events<crate::layer1::nanite_fabrication::ContainmentBreachEvent>>();
    }

    if !world
        .contains_resource::<Events<crate::layer2::trade::penal_contracts::PrisonerDiedEvent>>()
    {
        world.init_resource::<Events<crate::layer1::genetics::GeneSplicingEvent>>();
        world.init_resource::<Events<crate::layer1::genetics::GeneSplicingResultEvent>>();
        world.init_resource::<Events<crate::layer2::trade::penal_contracts::PrisonerDiedEvent>>();
    }

    if !world.contains_resource::<Events<crate::layer2::governance::RebellionEvent>>() {
        world.init_resource::<Events<crate::layer2::governance::RebellionEvent>>();
    }

    if !world.contains_resource::<Events<crate::layer1::environment::disasters::DisasterEvent>>() {
        world.init_resource::<Events<crate::layer1::environment::disasters::DisasterEvent>>();
    }

    if !world.contains_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>() {
        world.init_resource::<Events<crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent>>();
    }

    if !world.contains_resource::<Events<crate::layer1::agony_extract::HarvestAgonyExtractEvent>>()
    {
        world.init_resource::<Events<crate::layer1::agony_extract::HarvestAgonyExtractEvent>>();
        world.init_resource::<crate::layer1::agony_extract::AgonyExtractConfig>();
    }

    if !world.contains_resource::<Events<crate::layer2::moon_hermits::PopDesertedEvent>>() {
        world.init_resource::<Events<crate::layer2::moon_hermits::PopDesertedEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();
    }

    if !world.contains_resource::<Events<crate::layer1::environment::disasters::DisasterEvent>>() {
        world.init_resource::<Events<crate::layer1::environment::disasters::DisasterEvent>>();
    }

    if !world.contains_resource::<Events<crate::layer2::trade::biomass_tariff::TradeDeal>>() {
        world.init_resource::<Events<crate::layer2::trade::biomass_tariff::TradeDeal>>();
    }
    if !world.contains_resource::<Events<crate::layer1::geodetic::GolemFormedEvent>>() {
        world.init_resource::<Events<crate::layer1::geodetic::GolemFormedEvent>>();
    }

    if !world.contains_resource::<crate::layer3::market::GalacticMarket>() {
        world.init_resource::<crate::layer3::market::GalacticMarket>();
    }

    if !world.contains_resource::<Events<crate::layer2::events_new::reverse_quarantine::RefugeeFleetEvent>>() {
        world.init_resource::<Events<crate::layer2::events_new::reverse_quarantine::RefugeeFleetEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>() {
        world.init_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>();
        world.init_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>();
        world.init_resource::<Events<crate::layer3::planet::black_market_terraforming::RogueTerraformEvent>>();
        world.init_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>();
        world.init_resource::<Events<crate::layer3::market::quantum_famine::ExportDumpEvent>>();
        world
            .init_resource::<Events<crate::layer2::exploration::void_whispers::FleetReturnedEvent>>(
            );
        world.init_resource::<Events<crate::layer1::core::integration::PirateAmnestyEvent>>();
    }
    if !world
        .contains_resource::<Events<crate::layer2::navigation::stellar_weather::FleetDamagedEvent>>(
        )
    {
        world
            .init_resource::<Events<crate::layer2::navigation::stellar_weather::FleetDamagedEvent>>(
            );
    }

    if !world.contains_resource::<Events<crate::layer1::environment::ignition::SparkEvent>>() {
        world.init_resource::<Events<crate::layer1::environment::ignition::SparkEvent>>();
        world.init_resource::<Events<crate::layer1::environment::ignition::ExplosionEvent>>();
    }

    if !world.contains_resource::<Events<crate::layer1::environment::events::DebrisFallEvent>>() {
        world.init_resource::<Events<crate::layer1::environment::events::DebrisFallEvent>>();
        world.init_resource::<crate::layer3::council::GalacticCouncil>();
        world.init_resource::<crate::layer2::syzygy::SyzygyCycle>();
        world.init_resource::<crate::layer2::syzygy::PlanetaryGravity>();
        world.init_resource::<crate::layer2::syzygy::TidalForce>();
    }

    if !world.contains_resource::<Events<crate::layer1::logistics::mass_driver::LaunchEvent>>() {
        world.init_resource::<Events<crate::layer1::logistics::mass_driver::LaunchEvent>>();
        world.init_resource::<Events<crate::layer1::logistics::mass_driver::BombardmentEvent>>();
    }

    if !world.contains_resource::<crate::layer3::council::GalacticCouncil>() {
        world.init_resource::<crate::layer3::council::GalacticCouncil>();
    }

    if !world.contains_resource::<crate::layer2::syzygy::SyzygyCycle>() {
        world.init_resource::<crate::layer2::syzygy::SyzygyCycle>();
    }
    if !world.contains_resource::<crate::layer2::syzygy::PlanetaryGravity>() {
        world.init_resource::<crate::layer2::syzygy::PlanetaryGravity>();
    }
    if !world.contains_resource::<crate::layer2::syzygy::TidalForce>() {
        world.init_resource::<crate::layer2::syzygy::TidalForce>();
    }
    // Initialize Infinite Archive Resource (Spec 248)
    if !world.contains_resource::<crate::layer1::tech::infinite_archive::Archive>() {
        world.init_resource::<crate::layer1::tech::infinite_archive::Archive>();
    }

    if !world.contains_resource::<Events<HostileSpawnEvent>>() {
        world.init_resource::<Events<HostileSpawnEvent>>();
    }

    if !world.contains_resource::<crate::layer3::map::MapData>() {
        world.init_resource::<crate::layer3::map::MapData>();
    }
    if !world.contains_resource::<Events<crate::layer3::map::FleetArrivalEvent>>() {
        world.init_resource::<Events<crate::layer3::map::FleetArrivalEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer3::map::AnomalyDiscoveredEvent>>() {
        world.init_resource::<Events<crate::layer3::map::AnomalyDiscoveredEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy_reflection::EntityKilledEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy_reflection::FloraPlantedEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy_reflection::TraitChangedEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer2::cascade::LogisticsStrainedEvent>>() {
        world.init_resource::<Events<crate::layer2::cascade::LogisticsStrainedEvent>>();
    }
    if !world.contains_resource::<crate::layer3::physics::relativity::SimulationTime>() {
        world.init_resource::<crate::layer3::physics::relativity::SimulationTime>();
    }
    if !world.contains_resource::<Events<crate::layer2::cascade::DefenseWeakenedEvent>>() {
        world.init_resource::<Events<crate::layer2::cascade::DefenseWeakenedEvent>>();
    }
    // Add our schedule if not yet added
    {
        let schedules = world.resource::<Schedules>();
        if schedules.get(SimulationSchedule).is_none() {
            world.init_resource::<Events<crate::layer3::ghost_ships::EvaluateTransitEvent>>();
            world
                .init_resource::<Events<crate::layer3::ghost_ships::EvaluateLostShipReturnEvent>>();
            world.init_resource::<Events<crate::layer1::unseen_bureaucracy::PhantomShiftEvent>>();

            let schedule = build_simulation_schedule();
            world.add_schedule(schedule);
        }
    }

    world.run_schedule(SimulationSchedule);
    world.resource_mut::<SimulationTime>().tick += 1;
    world
        .resource_mut::<crate::layer3::physics::relativity::SimulationTime>()
        .tick += 1;
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
        world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::law::justice::CrimeCommittedEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::pop_memories::FamineEvent>>();
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
        world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();
        world.init_resource::<bevy::prelude::Events<crate::layer1::pop_memories::FamineEvent>>();
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
    fn test_schedule_runs_on_fresh_world() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        // Initialize Detection Risk for test
        world.init_resource::<crate::layer3::silence::DetectionRisk>();
        world.init_resource::<bevy::prelude::Events<crate::layer2::weather::StormImpactEvent>>();
        world
            .init_resource::<crate::layer1::environment::bio_acoustic_miasma::MiasmaRecordedSecret>(
            );
        world.init_resource::<bevy::prelude::Events<crate::layer2::skyhooks::LaunchIntent>>();
        world.init_resource::<Events<crate::layer2::moon_hermits::PopDesertedEvent>>();
        world.init_resource::<Events<crate::layer3::silence::HostileSpawnEvent>>();
        world.init_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::TogglePolicyEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::AccessDeniedEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::HackCentralHubEvent>>();
        world.init_resource::<Events<crate::layer1::administration::edicts::RevokePolicyEvent>>();
        world.init_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>();
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
        world.init_resource::<Events<crate::layer2::trade::escape_velocity::LaunchShipEvent>>();
        world.init_resource::<crate::layer2::trade::blockade::ColonyDebt>();

        world.init_resource::<Events<crate::layer3::events::debt_prison::BailoutOfferEvent>>();
        world.init_resource::<Events<crate::layer3::events::debt_prison::AcceptBailoutEvent>>();

        world.init_resource::<Events<crate::layer1::diplomacy::wards::WarDeclaredEvent>>();
        world.insert_resource(crate::layer1::diplomacy::wards::DiplomaticStanding {
            faction_relations: std::collections::HashMap::new(),
        });

        world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>();

        world.init_resource::<Events<crate::layer2::silent_mutiny::SensorGlitchEvent>>();
        world.init_resource::<Events<crate::layer1::nanite_fabrication::ContainmentBreachEvent>>();
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
        world.init_resource::<Events<crate::layer2::events_new::reverse_quarantine::RefugeeFleetEvent>>();
        world.init_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>();
        world.init_resource::<Events<crate::layer3::fleets::ColonyFoundedEvent>>();
        world.init_resource::<Events<crate::layer3::planet::black_market_terraforming::RogueTerraformEvent>>();
        world.init_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>();
        world.init_resource::<Events<crate::layer3::market::quantum_famine::ExportDumpEvent>>();
        world
            .init_resource::<Events<crate::layer2::exploration::void_whispers::FleetReturnedEvent>>(
            );
        world.init_resource::<Events<crate::layer1::core::integration::PirateAmnestyEvent>>();
        world.init_resource::<Events<crate::layer1::law::justice::CrimeCommittedEvent>>();
        world.init_resource::<Events<crate::layer1::pop_memories::FamineEvent>>();
        world
            .init_resource::<Events<crate::layer2::navigation::stellar_weather::FleetDamagedEvent>>(
            );
        world.init_resource::<Events<crate::layer1::environment::ignition::SparkEvent>>();
        world.init_resource::<Events<crate::layer1::environment::ignition::ExplosionEvent>>();
        world.init_resource::<Events<crate::layer1::environment::events::DebrisFallEvent>>();
        world.init_resource::<Events<crate::layer1::whispering_ore::MinedOreEvent>>();
        world.init_resource::<Events<crate::layer1::whispering_ore::MineSealedEvent>>();
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

        world.init_resource::<Events<crate::layer2::cascade::LogisticsStrainedEvent>>();
        world.init_resource::<Events<crate::layer2::cascade::DefenseWeakenedEvent>>();

        world.init_resource::<crate::layer3::physics::relativity::SimulationTime>();

        world.init_resource::<Events<crate::layer2::moon_hermits::PopDesertedEvent>>();

        world.init_resource::<Time>();
        world.init_resource::<Events<crate::layer2::primitives::InvasionEvent>>();

        world.init_resource::<Events<crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent>>();
        world.init_resource::<Events<crate::layer3::diplomacy::succession::SuccessionEvent>>();
        world.init_resource::<Events<crate::layer1::pop_memories::FamineEvent>>();
        world
            .init_resource::<Events<crate::layer3::diplomacy::succession::SuccessionCrisisEvent>>();

        world.init_resource::<Events<crate::layer3::ghost_ships::EvaluateTransitEvent>>();
        world.init_resource::<Events<crate::layer3::ghost_ships::EvaluateLostShipReturnEvent>>();
        world.init_resource::<Events<crate::layer1::unseen_bureaucracy::PhantomShiftEvent>>();

        let schedule = build_simulation_schedule();
        world.add_schedule(schedule);
        world.run_schedule(SimulationSchedule);

        // Should not panic — all systems run correctly on a fresh world
    }
}
