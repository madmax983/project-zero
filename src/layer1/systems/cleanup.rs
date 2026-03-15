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
            update_event_buffer::<crate::layer1::hazards::AmputationEvent>,
            update_event_buffer::<crate::layer1::geology::GeologicalEvent>,
            update_event_buffer::<crate::layer1::society::InvestigationEvent>,
            update_event_buffer::<crate::layer1::society::SuppressSocietyEvent>,
            update_event_buffer::<crate::layer1::medical::PatientTreated>,
            update_event_buffer::<crate::layer1::eureka::EurekaEvent>,
            update_event_buffer::<crate::layer1::items::UnequipEvent>,
            update_event_buffer::<crate::layer1::unrest::DenounceEvent>,
            update_event_buffer::<crate::layer1::volatile::ExplosionEvent>,
            update_event_buffer::<PossessEntityEvent>,
            update_event_buffer::<UnpossessEvent>,
            update_event_buffer::<crate::layer1::skills::XpGainEvent>,
            update_event_buffer::<crate::layer1::resources::MiningEvent>,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );

    schedule.add_systems(
        (
            update_event_buffer::<crate::layer1::ancestral_graves::SacrilegeEvent>,
            update_event_buffer::<crate::layer1::geology::tectonic::MegaQuakeEvent>,
            update_event_buffer::<crate::layer1::logistics::orbital_drop::OrbitalDropEvent>,
            update_event_buffer::<crate::layer1::spiteful_will::InheritanceEvent>,
            update_event_buffer::<crate::layer1::spiteful_will::OverrideWillEvent>,
            update_event_buffer::<crate::layer1::memory_core::ImplantMemoryCoreEvent>,
            update_event_buffer::<crate::layer1::memory_core::HarvestMemoryCoreEvent>,
            update_event_buffer::<crate::layer1::tech::machine_awakening::BotGlitchEvent>,
            handle_direct_input_system,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );
    schedule.add_systems(update_event_buffer::<crate::layer1::social::SocialInteractionEvent>.in_set(Layer1SystemSet::EventCleanup));
}
