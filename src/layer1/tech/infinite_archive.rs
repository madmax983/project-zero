use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::tech::{Tech, TechState};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataSize(pub f32);

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
            ..Default::default()});
        }
    }
}
