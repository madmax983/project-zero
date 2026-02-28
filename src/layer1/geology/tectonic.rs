use crate::layer1::resources::MiningEvent;
use crate::layer1::volatile::ExplosionEvent;
use bevy_ecs::prelude::*;

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
            dissipation_rate: 0.1,
        }
    }
}

#[derive(Event)]
pub struct MegaQuakeEvent;

pub fn update_stress_system(
    mut stress: ResMut<TectonicStress>,
    mut mining: EventReader<MiningEvent>,
    mut explosions: EventReader<ExplosionEvent>,
) {
    // 1. Add Stress
    for _ in mining.read() {
        stress.current += 0.5; // Tuning value
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
    mut geo_events: EventWriter<crate::layer1::geology::GeologicalEvent>,
) {
    if stress.current >= stress.threshold {
        quake_writer.send(MegaQuakeEvent);
        // Dispatch actual damage using GeologicalEvent
        geo_events.send(crate::layer1::geology::GeologicalEvent::Earthquake {
            center: crate::layer1::map::GridPosition { x: 50, y: 50 }, // Approximation since map size is not directly here, or we can just send multiple.
            magnitude: 15.0,                                           // MegaQuake is big
        });
        stress.current = 0.0; // Reset
    }
}
