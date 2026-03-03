use bevy_ecs::prelude::*;
use crate::layer1::skills::SkillType;

#[derive(Component, Debug, Clone)]
pub struct HypnoPod {
    pub target_skill: SkillType, // Default to something or allow "General" learning
    pub xp_rate: f32, // XP per tick
}

impl Default for HypnoPod {
    fn default() -> Self {
        Self {
            target_skill: SkillType::Mining,
            xp_rate: 10.0,
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct MentalFog {
    pub duration: f32, // Ticks or Seconds
    pub movement_penalty: f32, // 0.0 to 1.0 (multiplier)
    pub work_speed_penalty: f32,
}

#[derive(Component, Debug, Clone, Default)]
pub struct WasInHypnoPod;

use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;
use crate::layer1::skills::Skills;
use crate::layer1::actions::AssignedTo;
use crate::layer1::utility_types::{ActionType, AssignmentType};
use crate::layer1::PopAction;

pub fn hypno_sleep_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Skills, &mut Needs, &AssignedTo, &PopAction), With<Pop>>,
    pods: Query<&HypnoPod>,
) {
    for (entity, mut skills, mut needs, assigned, action) in pops.iter_mut() {
        if assigned.assignment_type == AssignmentType::HousingResident && action.current == ActionType::SatisfyRest {
            if let Ok(pod) = pods.get(assigned.entity) {
                // Grant XP
                skills.add_xp(pod.target_skill, pod.xp_rate);

                // Drain Hunger extra
                needs.hunger = (needs.hunger - 0.05).max(0.0); // Extra drain

                // Mark as being in hypno pod for waking up
                commands.entity(entity).insert(WasInHypnoPod);
            }
        }
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    query: Query<(Entity, &PopAction), With<WasInHypnoPod>>,
) {
    for (entity, action) in &query {
        // If they stop resting
        if action.current != ActionType::SatisfyRest {
            commands.entity(entity).remove::<WasInHypnoPod>();
            commands.entity(entity).insert(MentalFog {
                duration: 1000.0, // Long duration
                movement_penalty: 0.5, // Half speed
                work_speed_penalty: 0.5,
            });
        }
    }
}

pub fn update_mental_fog_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MentalFog)>,
) {
    for (entity, mut fog) in query.iter_mut() {
        fog.duration -= 1.0;
        if fog.duration <= 0.0 {
            commands.entity(entity).remove::<MentalFog>();
        }
    }
}
