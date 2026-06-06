use bevy_ecs::prelude::*;

#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum ObsessionType {
    DigHoles,
    StackChairs,
    #[default]
    HumCatchyTune,
}

#[derive(Component, PartialEq, Clone, Copy, Debug, Default)]
pub struct MemeticInfection {
    pub obsession_type: ObsessionType,
    pub intensity: f32,
}

#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct Quarantined;

use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer3::silence::DetectionRisk;

use crate::layer1::utility_eval_types::{PopEvalData, UtilityAIBuffer};
use crate::layer1::utility_types::ActionType;

pub fn evaluate_memetic_obsession(
    data: &PopEvalData,
    _buffer: &UtilityAIBuffer,
) -> Option<(ActionType, f32, Option<Entity>)> {
    if let Some(_infection) = &data.memetic_infection {
        // High utility to override normal needs (10.0)
        return Some((ActionType::MemeticObsession, 10.0, None));
    }
    None
}
pub fn process_parasitic_work_reduction(mut query: Query<&mut Morale, With<MemeticInfection>>) {
    for mut morale in query.iter_mut() {
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

pub fn parasitic_broadcast_risk_system(
    mut risk: ResMut<DetectionRisk>,
    query: Query<(), With<MemeticInfection>>,
) {
    // Every infected pop acts as a tiny antenna
    let infected_count = query.iter().count() as f32;
    risk.current_risk += infected_count * 0.1; // Accumulate risk
}
