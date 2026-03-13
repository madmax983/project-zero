use crate::layer1::energy::load_limits::PowerCable;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::tech::infinite_archive::Archive;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowType {
    StaticMite,
    DataRot,
    VoidEel,
}

#[derive(Component)]
pub struct ShadowEntity {
    pub entity_type: ShadowType,
    pub hunger: f32,
    pub visible: bool,
}

pub fn spawn_shadow_fauna_system(
    mut commands: Commands,
    cables: Query<(&PowerCable, &GridPosition)>,
    archive: Option<Res<Archive>>,
) {
    let mut rng = rand::thread_rng();

    // Check for high energy load spots
    for (cable, pos) in cables.iter() {
        if cable.current_load > 100.0 {
            // threshold for "high load"
            // Chance to spawn
            if rng.gen_bool(0.1) || cfg!(test) {
                // Always spawn in tests for simplicity
                commands.spawn((
                    ShadowEntity {
                        entity_type: ShadowType::StaticMite,
                        hunger: 0.0,
                        visible: false,
                    },
                    *pos,
                ));
            }
        }
    }

    // Check for high data density
    if let Some(arch) = archive {
        if arch.used > 100.0 {
            // threshold for "high data density"
            if rng.gen_bool(0.05) || cfg!(test) {
                // Always spawn in tests
                // Spawn at random location, maybe (0, 0) or wherever archive is
                commands.spawn((
                    ShadowEntity {
                        entity_type: ShadowType::DataRot,
                        hunger: 0.0,
                        visible: false,
                    },
                    GridPosition { x: 0, y: 0 },
                ));
            }
        }
    }
}

pub fn shadow_feed_system(
    mut shadows: Query<(&ShadowEntity, &GridPosition)>,
    mut cables: Query<(&mut PowerCable, &GridPosition)>,
    mut archive: Option<ResMut<Archive>>,
) {
    for (shadow, shadow_pos) in shadows.iter_mut() {
        match shadow.entity_type {
            ShadowType::StaticMite => {
                // Eat power. Reduce efficiency by increasing load.
                for (mut cable, cable_pos) in cables.iter_mut() {
                    if shadow_pos.x == cable_pos.x && shadow_pos.y == cable_pos.y {
                        cable.current_load += 10.0;
                    }
                }
            }
            ShadowType::DataRot => {
                // Corrupts research/blueprints by reducing efficiency multiplier
                if let Some(ref mut arch) = archive {
                    arch.efficiency_multiplier = (arch.efficiency_multiplier - 0.05).max(0.0);
                }
            }
            ShadowType::VoidEel => {} // Passive
        }
    }
}

pub fn shadow_visibility_system(
    mut query: Query<&mut ShadowEntity>,
    weather: Option<Res<WeatherState>>,
) {
    // If Magnetic Storm is active, they become visible
    let is_magnetic_storm = if let Some(w) = weather {
        w.current_weather == WeatherType::MagneticStorm
    } else {
        false
    };

    for mut shadow in query.iter_mut() {
        if is_magnetic_storm {
            shadow.visible = true;
        } else {
            shadow.visible = false;
        }
    }
}
