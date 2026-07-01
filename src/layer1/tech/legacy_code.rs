use crate::layer1::core::colony::ComputerCore;
use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug)]
pub struct SystemBloat {
    pub amount: f32,
    pub accumulation_rate: f32,
}

impl SystemBloat {
    pub fn efficiency(&self) -> f32 {
        (1.0 - (self.amount / 110.0)).max(0.1)
    }
}

#[derive(Component)]
pub struct ActionLatency {
    pub delay_ticks: u32,
}

#[derive(Component, PartialEq, Debug, Default)]
pub enum CoreStatus {
    #[default]
    Online,
    OfflineRebooting(u32),
}

#[derive(Event)]
pub struct ReformatCommand {
    pub core_entity: Entity,
}

pub fn accumulate_bloat_system(mut query: Query<(&mut SystemBloat, &mut CoreStatus)>) {
    for (mut bloat, mut status) in query.iter_mut() {
        match *status {
            CoreStatus::Online => {
                bloat.amount += bloat.accumulation_rate;
            }
            CoreStatus::OfflineRebooting(ref mut ticks) => {
                if *ticks > 0 {
                    *ticks -= 1;
                } else {
                    *status = CoreStatus::Online;
                }
            }
        }
    }
}

pub fn apply_latency_system(
    core_query: Query<&SystemBloat, With<ComputerCore>>,
    mut target_query: Query<&mut ActionLatency>, // Removing With<Turret> to make it general, but applies to things with ActionLatency
) {
    // Assuming one main core for MVP
    if let Ok(bloat) = core_query.get_single() {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let added_delay = (bloat.amount / 10.0) as u32; // Every 10 bloat = 1 tick delay

        for mut latency in target_query.iter_mut() {
            latency.delay_ticks = added_delay;
        }
    }
}

pub fn process_reformat_system(
    mut events: EventReader<ReformatCommand>,
    mut query: Query<(&mut SystemBloat, &mut CoreStatus)>,
) {
    for event in events.read() {
        if let Ok((mut bloat, mut status)) = query.get_mut(event.core_entity) {
            bloat.amount = 0.0;
            *status = CoreStatus::OfflineRebooting(600); // e.g. 600 ticks recovery
        }
    }
}
