use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug)]
pub struct Bloat {
    pub current: f32,      // 0-100
    pub rate: f32,         // e.g. 0.01 per tick
    pub reboot_ticks: u32, // 0 = Online, >0 = Rebooting
}

impl Bloat {
    pub fn efficiency(&self) -> f32 {
        // Non-linear decay
        // 0 bloat = 1.0
        // 100 bloat = 0.1 (min efficiency)
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
