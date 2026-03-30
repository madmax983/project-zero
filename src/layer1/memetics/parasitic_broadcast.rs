use bevy_ecs::prelude::*;
use crate::layer1::morale::{Morale, MoodModifier};
use crate::layer3::silence::DetectionRisk;
use crate::layer1::jobs::WorkEfficiency;

#[derive(Component, PartialEq)]
pub enum MemeticInfection {
    ParasiticBroadcast,
    // Future ones like 'The Silence' or 'Cult Belief'
}

pub fn process_parasitic_work_reduction(
    mut query: Query<(&mut Morale, &mut WorkEfficiency, &MemeticInfection)>,
) {
    for (mut morale, mut eff, infection) in query.iter_mut() {
        if *infection == MemeticInfection::ParasiticBroadcast {
            // High entertainment, horrible productivity
            morale.add_modifier(MoodModifier {
                label: "Catchy Tune".to_string(),
                value: 0.1, // Massive morale boost
                duration: 10,
            });
            eff.multiplier *= 0.5; // Halve their work output
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
