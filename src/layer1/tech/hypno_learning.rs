use bevy_ecs::prelude::*;
use crate::layer1::skills::SkillType;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;
use crate::layer1::utility_types::{PopAction, ActionType};
use crate::layer1::skills::Skills;

// Spec 255: Hypno-Learning Constants
pub const HYPNO_SLEEP_HUNGER_DRAIN: f32 = 5.0;
pub const MENTAL_FOG_DURATION: f32 = 1000.0;
pub const MENTAL_FOG_MOVEMENT_PENALTY: f32 = 0.5;
pub const MENTAL_FOG_WORK_SPEED_PENALTY: f32 = 0.5;

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

#[derive(Component, Debug, Clone)]
pub struct HypnoSleeping {
    pub bed_entity: Entity,
}

#[derive(Component, Debug, Clone, Default)]
pub struct MentalFog {
    pub duration: f32, // Ticks or Seconds
    pub movement_penalty: f32, // 0.0 to 1.0 (multiplier)
    pub work_speed_penalty: f32,
}

pub fn hypno_sleep_system(
    mut pops: Query<(&mut Skills, &mut Needs, &HypnoSleeping, Option<&PopAction>), With<Pop>>,
    pods: Query<&HypnoPod>,
) {
    for (mut skills, mut needs, hypno_sleeping, action) in pops.iter_mut() {
        let mut is_sleeping = false;
        if let Some(act) = action {
            if act.current == ActionType::SatisfyRest {
                is_sleeping = true;
            }
        } else {
            // For testing convenience when PopAction isn't required.
            is_sleeping = true;
        }

        if is_sleeping {
            if let Ok(pod) = pods.get(hypno_sleeping.bed_entity) {
                // Grant XP
                skills.add_xp(pod.target_skill, pod.xp_rate);

                // Drain Hunger extra
                needs.hunger = (needs.hunger - HYPNO_SLEEP_HUNGER_DRAIN).max(0.0); // Ensure noticeable drop
            }
        }
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    pops: Query<(Entity, Option<&PopAction>, &HypnoSleeping), With<Pop>>,
    pods: Query<&HypnoPod>,
) {
    for (entity, action, hypno_sleeping) in pops.iter() {
        let mut woke_up = false;
        if let Some(act) = action {
            if act.current != ActionType::SatisfyRest {
                woke_up = true;
            }
        } else {
             // For testing convenience.
             woke_up = true;
        }

        if woke_up {
            if pods.get(hypno_sleeping.bed_entity).is_ok() {
                // Apply Fog
                commands.entity(entity).insert(MentalFog {
                    duration: MENTAL_FOG_DURATION,
                    movement_penalty: MENTAL_FOG_MOVEMENT_PENALTY,
                    work_speed_penalty: MENTAL_FOG_WORK_SPEED_PENALTY,
                });
            }
            commands.entity(entity).remove::<HypnoSleeping>();
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
