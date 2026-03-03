use super::{Layer1SystemSet, update_event_buffer};
use crate::layer1::direct_link::{PossessEntityEvent, UnpossessEvent, handle_direct_input_system};
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
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );

    schedule.add_systems(
        (
            update_event_buffer::<crate::layer1::medical::PatientTreated>,
            update_event_buffer::<crate::layer1::eureka::EurekaEvent>,
            update_event_buffer::<crate::layer1::items::UnequipEvent>,
            update_event_buffer::<crate::layer1::unrest::DenounceEvent>,
            update_event_buffer::<crate::layer1::volatile::ExplosionEvent>,
            update_event_buffer::<crate::layer1::geology::tectonic::MiningEvent>,
            update_event_buffer::<crate::layer1::geology::tectonic::MegaQuakeEvent>,
            update_event_buffer::<PossessEntityEvent>,
            update_event_buffer::<UnpossessEvent>,
            update_event_buffer::<crate::layer1::skills::XpGainEvent>,
            handle_direct_input_system,
        )
            .in_set(Layer1SystemSet::EventCleanup),
    );
}
