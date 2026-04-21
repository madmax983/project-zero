use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug)]
pub struct Bloat {
    pub current: f32, // 0-100
    pub rate: f32,    // e.g. 0.01 per tick
}

impl Bloat {
    pub fn efficiency(&self) -> f32 {
        // Non-linear decay
        // 0 bloat = 1.0
        // 100 bloat = 0.1 (min efficiency)
        (1.0 - (self.current / 110.0)).max(0.1)
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum SystemStatus {
    Online,
    Rebooting(u32), // Ticks remaining
}

pub fn update_bloat_system(mut query: Query<(&mut Bloat, &mut SystemStatus)>) {
    for (mut bloat, mut status) in query.iter_mut() {
        match *status {
            SystemStatus::Online => {
                bloat.current = (bloat.current + bloat.rate).min(100.0);
            }
            SystemStatus::Rebooting(ref mut ticks) => {
                if *ticks > 0 {
                    *ticks -= 1;
                } else {
                    // Done
                    bloat.current = 0.0;
                    *status = SystemStatus::Online;
                }
            }
        }
    }
}
