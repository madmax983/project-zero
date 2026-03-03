use bevy_ecs::prelude::*;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::utility_types::PopAction;
use crate::layer1::AssignedTo;
use crate::layer1::energy::PowerConsumer;

#[derive(Component, Debug, Clone)]
pub struct HypnoPod {
    pub target_skill: SkillType,
    pub xp_rate: f32,
}

impl Default for HypnoPod {
    fn default() -> Self {
        Self {
            target_skill: SkillType::Mining,
            xp_rate: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct MentalFog {
    pub duration: f32,
    pub movement_penalty: f32,
    pub work_speed_penalty: f32,
}

#[derive(Component)]
pub struct SleepingInHypnoPod;

pub fn hypno_sleep_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Skills, &mut Needs, &PopAction, &AssignedTo, Option<&SleepingInHypnoPod>), With<Pop>>,
    pods: Query<(&HypnoPod, Option<&PowerConsumer>)>,
) {
    for (entity, mut skills, mut needs, action, assignment, sleep_state) in pops.iter_mut() {
        if action.current == crate::layer1::utility_types::ActionType::SatisfyRest {
            if let Ok((pod, power_opt)) = pods.get(assignment.entity) {
                // Only grant benefits if we have power
                if power_opt.map_or(true, |p| p.active) {
                    skills.add_xp(pod.target_skill, pod.xp_rate);
                    needs.hunger = (needs.hunger - 0.5).max(0.0);
                }
                if sleep_state.is_none() {
                    commands.entity(entity).insert(SleepingInHypnoPod);
                }
            }
        }
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    pops: Query<(Entity, &PopAction, &SleepingInHypnoPod)>,
) {
    for (entity, action, _) in pops.iter() {
        if action.current != crate::layer1::utility_types::ActionType::SatisfyRest {
            commands.entity(entity).remove::<SleepingInHypnoPod>();
            commands.entity(entity).insert(MentalFog {
                duration: 1000.0,
                movement_penalty: 0.5,
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

pub fn apply_mental_fog_speed_system(
    mut query: Query<(&mut crate::layer1::pop::Speed, &MentalFog)>,
) {
    for (mut speed, fog) in query.iter_mut() {
        speed.current *= fog.movement_penalty;
    }
}
