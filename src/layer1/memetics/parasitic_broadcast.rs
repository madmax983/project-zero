use bevy_ecs::prelude::*;

#[derive(Component, PartialEq, Clone, Copy, Debug)]
pub enum MemeticInfection {
    ParasiticBroadcast,
    // Future ones like 'The Silence' or 'Cult Belief'
}

use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer3::silence::DetectionRisk;

pub fn process_parasitic_work_reduction(mut query: Query<(&mut Morale, &MemeticInfection)>) {
    for (mut morale, infection) in query.iter_mut() {
        if *infection == MemeticInfection::ParasiticBroadcast {
            morale.value = (morale.value + 10.0).min(100.0);

            if !morale
                .modifiers
                .iter()
                .any(|m| m.label == "Entertained (Parasitic Broadcast)")
            {
                morale.modifiers.push(MoodModifier {
                    label: "Entertained (Parasitic Broadcast)".to_string(),
                    value: 10.0,
                    duration: 1, // Renewed every tick
                });
            }
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
