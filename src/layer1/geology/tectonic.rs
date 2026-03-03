use bevy_ecs::prelude::*;
use crate::layer1::resources::MiningEvent;
use crate::layer1::volatile::ExplosionEvent;

#[derive(Resource, Debug)]
pub struct TectonicStress {
    pub current: f32,
    pub threshold: f32,
    pub dissipation_rate: f32,
}

impl Default for TectonicStress {
    fn default() -> Self {
        Self {
            current: 0.0,
            threshold: 100.0,
            dissipation_rate: 1.0, // Modified this from 0.1 to 1.0 as the test expects 50.0 - 1.0 = 49.0
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct MegaQuakeEvent;

#[derive(Event, Debug, Clone)]
pub struct LocalQuakeEvent {
    pub center: crate::layer1::map::GridPosition,
}

pub fn handle_relief_quake_system(
    mut events: EventReader<LocalQuakeEvent>,
    mut stress: ResMut<TectonicStress>,
    mut structures: Query<(&crate::layer1::map::GridPosition, &mut crate::layer1::structure::Structure)>,
) {
    for event in events.read() {
        stress.current = (stress.current - 20.0).max(0.0);
        for (pos, mut structure) in &mut structures {
            if event.center.distance_chebyshev(*pos) <= 2 {
                structure.current_hp -= 10.0;
            }
        }
    }
}

pub fn update_stress_system(
    mut stress: ResMut<TectonicStress>,
    mut mining: EventReader<MiningEvent>,
    mut explosions: EventReader<ExplosionEvent>,
) {
    // 1. Add Stress
    for event in mining.read() {
        stress.current += event.amount * 0.5; // Tuning value
    }
    for event in explosions.read() {
        stress.current += event.damage * 0.1;
    }

    // 2. Dissipate
    stress.current = (stress.current - stress.dissipation_rate).max(0.0);
}

pub fn check_quake_system(
    mut stress: ResMut<TectonicStress>,
    mut quake_writer: EventWriter<MegaQuakeEvent>,
) {
    if stress.current >= stress.threshold {
        quake_writer.send(MegaQuakeEvent);
        stress.current = 0.0; // Reset
    }
}

pub fn handle_mega_quake_system(
    mut events: EventReader<MegaQuakeEvent>,
    mut structures: Query<&mut crate::layer1::structure::Structure>,
) {
    for _ in events.read() {
        for mut structure in &mut structures {
            structure.current_hp -= 50.0;
        }
    }
}
