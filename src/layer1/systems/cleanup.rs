use super::{update_event_buffer, Layer1SystemSet};
use crate::layer1::direct_link::{handle_direct_input_system, PossessEntityEvent, UnpossessEvent};
use crate::layer1::pop::{PopBorn, PopDied};
use crate::layer1::*;
use bevy_ecs::prelude::*;

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            update_event_buffer::<AddChronicleEvent>,
            update_event_buffer::<AffinityChange>,
            update_event_buffer::<PopDied>,
            update_event_buffer::<PopBorn>,
            update_event_buffer::<crate::layer1::structural_integrity::StructureCollapsed>,
            update_event_buffer::<crate::layer1::heirloom::RetrogradeEngineeringEvent>,
            update_event_buffer::<crate::layer1::energy::GridOverloadEvent>,
            update_event_buffer::<crate::layer1::environment::hazards::AmputationEvent>,
            update_event_buffer::<crate::layer1::geology::GeologicalEvent>,
            update_event_buffer::<crate::layer1::society::InvestigationEvent>,
            update_event_buffer::<crate::layer1::society::SuppressSocietyEvent>,
            update_event_buffer::<crate::layer1::medical::PatientTreated>,
            update_event_buffer::<crate::layer1::eureka::EurekaEvent>,
            update_event_buffer::<crate::layer1::items::UnequipEvent>,
            update_event_buffer::<crate::layer1::unrest::DenounceEvent>,
            update_event_buffer::<crate::layer1::environment::volatile::ExplosionEvent>,
            update_event_buffer::<PossessEntityEvent>,
            update_event_buffer::<UnpossessEvent>,
            update_event_buffer::<crate::layer1::skills::XpGainEvent>,
            update_event_buffer::<crate::layer1::resources::MiningEvent>,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );

    schedule.add_systems(
        (
            update_event_buffer::<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>,
            update_event_buffer::<crate::layer1::ancestral_graves::SacrilegeEvent>,
            update_event_buffer::<crate::layer1::geology::tectonic::MegaQuakeEvent>,
            update_event_buffer::<crate::layer1::logistics::orbital_drop::OrbitalDropEvent>,
            update_event_buffer::<crate::layer1::geodetic::GolemFormedEvent>,
            update_event_buffer::<crate::layer1::drone::DroneDisconnectedEvent>,
            update_event_buffer::<crate::layer1::social::gossip_economy::GossipEvent>,
            update_event_buffer::<crate::layer1::social::gossip_economy::BrokerPurchaseEvent>,
            update_event_buffer::<crate::layer1::spiteful_will::InheritanceEvent>,
            update_event_buffer::<crate::layer1::spiteful_will::OverrideWillEvent>,
            update_event_buffer::<crate::layer1::memory_core::ImplantMemoryCoreEvent>,
            update_event_buffer::<crate::layer1::memory_core::HarvestMemoryCoreEvent>,
            update_event_buffer::<crate::layer1::tech::machine_awakening::BotGlitchEvent>,
            update_event_buffer::<crate::layer2::trade::blockade::TradeShipArrivalEvent>,
            update_event_buffer::<crate::layer3::events::debt_prison::BailoutOfferEvent>,
            update_event_buffer::<crate::layer3::events::debt_prison::AcceptBailoutEvent>,
            update_event_buffer::<crate::layer1::genetics::GeneSplicingResultEvent>,
            update_event_buffer::<crate::layer1::genetics::CropMutationEvent>,
            update_event_buffer::<crate::layer1::economy::remittances::MigrantArrivalEvent>,
            handle_direct_input_system,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );

    schedule.add_systems(
        (
            update_event_buffer::<crate::layer1::shipbreaking::SpawnCrashedShipEvent>,
            update_event_buffer::<crate::layer1::shipbreaking::MineEvent>,
            update_event_buffer::<crate::layer2::moon_hermits::PopDesertedEvent>,
            update_event_buffer::<crate::layer1::grafting::GraftBuildingEvent>,
            update_event_buffer::<crate::layer3::market::quantum_famine::MarketPanicEvent>,
            update_event_buffer::<crate::layer3::market::quantum_famine::ExportDumpEvent>,
            update_event_buffer::<crate::layer2::exploration::void_whispers::FleetReturnedEvent>,
            update_event_buffer::<crate::layer1::agony_extract::HarvestAgonyExtractEvent>,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );

    schedule.add_systems(
        (
            update_event_buffer::<crate::layer1::diplomacy::wards::WarDeclaredEvent>,
            update_event_buffer::<crate::layer1::environment::orbital_tether::TetherSnapEvent>,
            update_event_buffer::<crate::layer1::architecture::BiomimeticShiftEvent>,
            update_event_buffer::<crate::layer1::digital_immortality::MindUploadEvent>,
            update_event_buffer::<crate::layer1::digital_immortality::GhostHackEvent>,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );

    schedule.add_systems(
        (
            update_event_buffer::<crate::layer1::environment::ephemeral_moons::MoonCapturedEvent>,
            update_event_buffer::<crate::layer1::environment::ephemeral_moons::MoonEjectedEvent>,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );

    schedule.add_systems(
        (
            update_event_buffer::<crate::layer3::market::ephemeral_market::MarketSpawnEvent>,
            update_event_buffer::<crate::layer3::market::ephemeral_market::MarketTradeEvent>,
            update_event_buffer::<crate::layer3::market::ephemeral_market::MarketTradeFailedEvent>,
            update_event_buffer::<crate::layer1::nature::megafauna_terrain::AwakenTitanEvent>,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );
}
