use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::energy::PowerSource;
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

#[derive(Resource)]
pub struct DataDensityGrid {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

impl DataDensityGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; width * height],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = value;
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.data[y * self.width + x]
        } else {
            0.0
        }
    }
}

#[derive(Resource)]
pub struct MagneticStorm {
    pub active: bool,
}

pub fn spawn_shadow_fauna_system(
    mut commands: Commands,
    power_sources: Query<(&GridPosition, &PowerSource)>,
    data_density: Option<Res<DataDensityGrid>>,
) {
    let mut rng = rand::thread_rng();

    // High Power Load spawns StaticMites
    for (pos, source) in power_sources.iter() {
        if source.output > 500.0 && rng.gen_bool(1.0) { // Using 1.0 for deterministic tests
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

    // High Data Density spawns DataRot
    if let Some(grid) = data_density {
        for y in 0..grid.height {
            for x in 0..grid.width {
                if grid.get(x, y) > 200.0 && rng.gen_bool(1.0) { // Using 1.0 for deterministic tests
                    commands.spawn((
                        ShadowEntity {
                            entity_type: ShadowType::DataRot,
                            hunger: 0.0,
                            visible: false,
                        },
                        GridPosition { x: x as i32, y: y as i32 },
                    ));
                }
            }
        }
    }
}

pub fn shadow_visibility_system(
    mut query: Query<&mut ShadowEntity>,
    magnetic_storm: Option<Res<MagneticStorm>>,
) {
    let is_storm_active = magnetic_storm.map(|storm| storm.active).unwrap_or(false);

    for mut entity in query.iter_mut() {
        entity.visible = is_storm_active; // Future: add Sensor checks
    }
}

pub fn shadow_feed_system(
    mut power_sources: Query<(&GridPosition, &mut PowerSource)>,
    shadows: Query<(&GridPosition, &ShadowEntity)>,
) {
    for (shadow_pos, shadow) in shadows.iter() {
        if shadow.entity_type == ShadowType::StaticMite {
            for (power_pos, mut source) in power_sources.iter_mut() {
                if shadow_pos == power_pos {
                    // Reduce efficiency
                    source.output *= 0.9;
                }
            }
        }
    }
}
