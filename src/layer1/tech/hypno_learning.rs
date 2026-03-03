use crate::layer1::skills::SkillType;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct HypnoPod {
    pub target_skill: SkillType,
    pub xp_rate: f32, // XP per tick
}

impl Default for HypnoPod {
    fn default() -> Self {
        Self {
            target_skill: SkillType::Mining,
            xp_rate: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct MentalFog {
    pub duration: f32,         // Ticks
    pub movement_penalty: f32, // 0.0 to 1.0 (multiplier)
    pub work_speed_penalty: f32,
}

impl Default for MentalFog {
    fn default() -> Self {
        Self {
            duration: 1000.0,
            movement_penalty: 0.5,
            work_speed_penalty: 0.5,
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct HypnoSleepMarker;

use crate::layer1::needs::Needs;
use crate::layer1::pop::Speed;
use crate::layer1::skills::Skills;
use crate::layer1::utility_types::{ActionType, AssignmentType, PopAction};
use crate::layer1::AssignedTo;

#[allow(clippy::type_complexity)]
pub fn hypno_sleep_system(
    mut commands: Commands,
    mut pops: Query<
        (
            Entity,
            &mut Skills,
            &mut Needs,
            &PopAction,
            &AssignedTo,
            Option<&HypnoSleepMarker>,
        ),
        With<crate::layer1::pop::Pop>,
    >,
    pods: Query<&HypnoPod>,
) {
    for (entity, mut skills, mut needs, action, assigned, marker_opt) in pops.iter_mut() {
        if action.current == ActionType::SatisfyRest
            && assigned.assignment_type == AssignmentType::HousingResident
        {
            if let Ok(pod) = pods.get(assigned.entity) {
                // Add marker if not already present
                if marker_opt.is_none() {
                    commands.entity(entity).insert(HypnoSleepMarker);
                }

                // Grant XP
                skills.add_xp(pod.target_skill, pod.xp_rate);

                // Drain Hunger extra
                needs.hunger = (needs.hunger - 0.5).max(0.0);
            }
        }
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &PopAction), With<HypnoSleepMarker>>,
) {
    for (entity, action) in pops.iter_mut() {
        if action.current != ActionType::SatisfyRest {
            commands.entity(entity).remove::<HypnoSleepMarker>();
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

pub fn apply_mental_fog_speed_system(mut pops: Query<(&mut Speed, &MentalFog)>) {
    for (mut speed, fog) in pops.iter_mut() {
        speed.current *= fog.movement_penalty;
    }
}
