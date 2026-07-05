use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::tech::{Tech, TechState};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use strum::IntoEnumIterator;

#[derive(Resource, Default, Debug)]
pub struct Archive {
    pub capacity: f32,
    pub used: f32,
    pub efficiency_multiplier: f32,
}

pub fn update_efficiency_system(mut archive: ResMut<Archive>, tech_state: Res<TechState>) {
    archive.capacity = tech_state.total_capacity;
    archive.used = tech_state.used_capacity;

    if archive.capacity > f32::EPSILON {
        let ratio = archive.used / archive.capacity;
        // Linear decay: 0% usage = 100% speed, 100% usage = 50% speed.
        archive.efficiency_multiplier = (1.0 - (ratio * 0.5)).max(0.0);
    } else {
        // No server capacity
        if archive.used > 0.0 {
            archive.efficiency_multiplier = 0.0;
        } else {
            archive.efficiency_multiplier = 1.0;
        }
    }
}

pub fn purge_tech(world: &mut World, tech_label: &str) {
    // Find the Tech enum from label by iterating over all variants.
    // This is robust against new Techs being added.
    let tech_to_purge = Tech::iter().find(|t| t.label() == tech_label);

    if let Some(tech) = tech_to_purge {
        let mut tech_state = world.resource_mut::<TechState>();
        if tech_state.techs.contains_key(&tech) {
            // Remove from techs map entirely (Forgotten)
            tech_state.techs.remove(&tech);
            // Re-run corruption check to update used capacity
            tech_state.update_corruption();

            if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                log.add(format!("Tech Purged: {}", tech_label));
            }

            world.send_event(AddChronicleEvent {
                text: format!(
                    "We have forgotten the secrets of {} to make room for new knowledge.",
                    tech_label
                ),
                importance: EventImportance::Standard,
            });
        }
    }
}

#[derive(Component)]
pub struct ServerRack {
    pub capacity: u32,
}

#[derive(Component, Default)]
pub struct DataStorage {
    pub used: u32,
}

#[derive(Resource, Default)]
pub struct TotalData(pub u32);

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum TechId {
    Laser,
    SteamEngine,
}

#[derive(Resource, Default)]
pub struct UnlockedTechs(pub Vec<TechId>);

#[derive(Resource)]
pub struct ResearchProgress {
    pub current_tech: TechId,
    pub progress: f32,
}

#[derive(Event)]
pub struct DeleteTechDataEvent(pub TechId);

pub fn research_tick_system(
    progress: Option<ResMut<ResearchProgress>>,
    total_data: Option<Res<TotalData>>,
) {
    if let Some(mut prog) = progress {
        let data_val = total_data.map(|d| d.0).unwrap_or(0);
        let penalty = 1.0 - (data_val as f32 / 1000.0).clamp(0.0, 0.9);
        prog.progress += 1.0 * penalty;
    }
}

pub fn update_storage_system(
    mut query: Query<(Entity, &ServerRack, Option<&mut DataStorage>)>,
    mut commands: Commands,
) {
    for (entity, _rack, storage) in query.iter_mut() {
        if let Some(mut s) = storage {
            s.used += 1;
        } else {
            commands.entity(entity).insert(DataStorage { used: 1 });
        }
    }
}

pub fn delete_tech_observer(
    trigger: Trigger<DeleteTechDataEvent>,
    unlocked: Option<ResMut<UnlockedTechs>>,
    total_data: Option<ResMut<TotalData>>,
) {
    let event = trigger.event();
    if let (Some(mut unl), Some(mut td)) = (unlocked, total_data) {
        if let Some(pos) = unl.0.iter().position(|t| *t == event.0) {
            unl.0.remove(pos);
            td.0 = td.0.saturating_sub(100);
        }
    }
}

pub fn delete_tech_system(
    mut events: EventReader<DeleteTechDataEvent>,
    unlocked: Option<ResMut<UnlockedTechs>>,
    total_data: Option<ResMut<TotalData>>,
) {
    if let (Some(mut unl), Some(mut td)) = (unlocked, total_data) {
        for event in events.read() {
            if let Some(pos) = unl.0.iter().position(|t| *t == event.0) {
                unl.0.remove(pos);
                td.0 = td.0.saturating_sub(100);
            }
        }
    }
}
