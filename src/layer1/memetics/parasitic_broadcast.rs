use bevy_ecs::prelude::*;

#[derive(Component, PartialEq, Clone, Copy, Debug)]
pub enum MemeticInfection {
    ParasiticBroadcast,
    // Future ones like 'The Silence' or 'Cult Belief'
}

use crate::layer1::morale::{MoodModifier, Morale};

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
