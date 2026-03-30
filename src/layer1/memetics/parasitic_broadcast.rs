// src/layer1/memetics/parasitic_broadcast.rs

use bevy_ecs::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Speed;
use crate::layer3::silence::DetectionRisk;

#[derive(Component, PartialEq, Debug)]
pub enum MemeticInfection {
    ParasiticBroadcast,
    // Future ones like 'The Silence' or 'Cult Belief'
}

pub fn process_parasitic_work_reduction(
    mut query: Query<(&mut Needs, &mut Speed, &MemeticInfection)>,
) {
    for (mut needs, mut speed, infection) in query.iter_mut() {
        if *infection == MemeticInfection::ParasiticBroadcast {
            // High entertainment, horrible productivity
            needs.leisure = (needs.leisure + 0.5).min(1.0);
            speed.current *= 0.5; // Halve their work output
        }
    }
}

pub fn parasitic_broadcast_risk_system(
    mut risk: ResMut<DetectionRisk>,
    query: Query<(), With<MemeticInfection>>,
) {
    // Every infected pop acts as a tiny antenna
    let infected_count = query.iter().count() as f32;
    risk.current_risk += infected_count * 0.1; // Accumulate risk
}
