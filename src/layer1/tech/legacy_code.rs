use crate::layer1::architecture::turret::Turret;
use bevy_ecs::prelude::*;

// Keep `Bloat` and `update_bloat_system` around since other parts of the codebase
// currently use them (like the library system we saw earlier).
#[derive(Component, Default, Debug)]
pub struct Bloat {
    pub current: f32,      // 0-100
    pub rate: f32,         // e.g. 0.01 per tick
    pub reboot_ticks: u32, // 0 = Online, >0 = Rebooting
}

impl Bloat {
    pub fn efficiency(&self) -> f32 {
        (1.0 - (self.current / 110.0)).max(0.1)
    }
}

pub fn update_bloat_system(mut query: Query<&mut Bloat>) {
    for mut bloat in query.iter_mut() {
        if bloat.reboot_ticks > 0 {
            bloat.reboot_ticks -= 1;
            if bloat.reboot_ticks == 0 {
                bloat.current = 0.0;
            }
        } else {
            bloat.current = (bloat.current + bloat.rate).min(100.0);
        }
    }
}

// -------------------------------------------------------------
// New spec 1017: Legacy Code Components
// -------------------------------------------------------------

#[derive(Component)]
pub struct ComputerCore;

#[derive(Component)]
pub struct SystemBloat {
    pub amount: f32,
    pub accumulation_rate: f32,
}

#[derive(Component)]
pub struct ActionLatency {
    pub delay_ticks: u32,
}

#[derive(Component, PartialEq, Debug)]
pub enum CoreStatus {
    Online,
    OfflineRebooting,
}

#[derive(Event)]
pub struct ReformatCommand {
    pub core_entity: Entity,
}

pub fn accumulate_bloat_system(mut query: Query<(&mut SystemBloat, &CoreStatus)>) {
    for (mut bloat, status) in query.iter_mut() {
        if *status == CoreStatus::Online {
            bloat.amount += bloat.accumulation_rate;
        }
    }
}

pub fn apply_latency_system(
    core_query: Query<&SystemBloat, With<ComputerCore>>,
    mut target_query: Query<&mut ActionLatency, With<Turret>>,
) {
    // Assuming one main core for MVP
    if let Ok(bloat) = core_query.get_single() {
        let added_delay = (bloat.amount / 10.0) as u32;

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
            *status = CoreStatus::OfflineRebooting;
        }
    }
}
