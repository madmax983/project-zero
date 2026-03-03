use bevy_ecs::prelude::*;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::needs::Needs;
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::layer1::actions::AssignedTo;
use crate::layer1::pop::Pop;
use crate::layer1::energy::PowerConsumer;

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

#[derive(Component, Default)]
pub struct SleepingInHypnoPod;


pub fn hypno_sleep_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Skills, &mut Needs, &AssignedTo, &PopAction), With<Pop>>,
    pods: Query<(&HypnoPod, Option<&PowerConsumer>)>,
) {
    for (entity, mut skills, mut needs, assigned, action) in &mut pops {
        if action.current == ActionType::SatisfyRest {
            if let Ok((pod, power_opt)) = pods.get(assigned.entity) {
                if power_opt.is_none_or(|p| p.active) {
                    // Grant XP
                    skills.add_xp(pod.target_skill, pod.xp_rate);

                    // Drain Hunger extra
                    needs.hunger = (needs.hunger - 0.5).max(0.0); // Arbitrary drain rate
                }

                commands.entity(entity).insert(SleepingInHypnoPod);
            }
        }
    }
}


pub fn wake_up_hypno_system(
    mut commands: Commands,
    mut query: Query<(Entity, &PopAction), With<SleepingInHypnoPod>>,
) {
    for (entity, action) in &mut query {
        if action.current != ActionType::SatisfyRest {
            // Apply Fog
            commands.entity(entity).insert(MentalFog {
                duration: 1000.0, // Long duration
                movement_penalty: 0.5, // Half speed
                work_speed_penalty: 0.5,
            });
            commands.entity(entity).remove::<SleepingInHypnoPod>();
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
