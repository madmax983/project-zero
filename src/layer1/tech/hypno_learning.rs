use bevy_ecs::prelude::*;

use crate::layer1::balance::TICKS_PER_YEAR;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::housing::Housing;
use crate::layer1::lifecycle::Age;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::utility_types::{ActionType, PopAction};

/// A high-tech alternative to standard beds that grants XP while sleeping.
#[derive(Component, Debug, Clone)]
pub struct HypnoPod {
    /// The skill to train.
    pub target_skill: SkillType,
    /// The amount of XP granted per tick.
    pub xp_rate: f32,
}

impl Default for HypnoPod {
    fn default() -> Self {
        Self {
            target_skill: SkillType::Mining,
            xp_rate: 0.1,
        }
    }
}

/// Applies a penalty to movement and work speed after waking from a HypnoPod.
#[derive(Component, Debug, Clone)]
pub struct MentalFog {
    /// Duration remaining in ticks.
    pub duration: f32,
    /// Movement speed multiplier (e.g. 0.5 = half speed).
    pub movement_penalty: f32,
    /// Work speed multiplier.
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

/// Marker component indicating a pop is currently sleeping in a specific HypnoPod.
#[derive(Component, Debug, Clone, Copy)]
pub struct SleepingInHypnoPod(pub Entity);

/// Processes sleeping in HypnoPods: grants XP, drains extra hunger.
pub fn hypno_sleep_system(
    mut commands: Commands,
    pods: Query<(Entity, &HypnoPod, Option<&PowerConsumer>), With<Housing>>,
    mut pops: Query<
        (
            Entity,
            &mut Skills,
            &mut Needs,
            &mut Traits,
            &Age,
            &PopAction,
            Option<&MovementTarget>,
            Option<&SleepingInHypnoPod>,
        ),
        (With<Pop>, With<AtTarget>),
    >,
) {
    for (pod_entity, pod, power) in pods.iter() {
        // HypnoPod must be powered
        if power.is_some_and(|p| !p.active) {
            continue;
        }

        // Find pops that are sleeping here
        for (pop_entity, mut skills, mut needs, mut traits, age, action, mt_opt, sleeping_opt) in pops.iter_mut() {
            if action.current == ActionType::SatisfyRest {
                let is_at_this_pod = if let Some(sleeping) = sleeping_opt {
                    sleeping.0 == pod_entity
                } else if let Some(mt) = mt_opt {
                    mt.target_entity == pod_entity
                } else {
                    false
                };

                if is_at_this_pod {
                    // Pop is sleeping in this pod. Add marker if they don't have it.
                    if sleeping_opt.is_none() {
                        commands.entity(pop_entity).insert(SleepingInHypnoPod(pod_entity));
                    }

                    // Check age
                    #[allow(clippy::cast_precision_loss)]
                    let years = age.ticks_alive as f32 / TICKS_PER_YEAR as f32;
                    if years < 18.0 {
                        // Child gets Traumatized instead of XP
                        traits.add(Trait::Traumatized);
                    } else {
                        // Adult gets XP
                        skills.add_xp(pod.target_skill, pod.xp_rate);
                    }

                    // Drain Hunger extra
                    // Normal rest decay handles rest need. We add a severe hunger drain.
                    needs.hunger = (needs.hunger - 0.005).max(0.0);
                }
            }
        }
    }
}

/// Detects when a pop wakes up from a HypnoPod (or if it loses power) and applies Mental Fog.
pub fn wake_up_hypno_system(
    mut commands: Commands,
    pops: Query<(Entity, &SleepingInHypnoPod, &PopAction)>,
    pods: Query<Option<&PowerConsumer>, With<HypnoPod>>,
) {
    for (pop_entity, sleeping, action) in pops.iter() {
        let pod_entity = sleeping.0;

        // Check if pod exists and is powered
        let pod_active = if let Ok(power_opt) = pods.get(pod_entity) {
            power_opt.is_none_or(|p| p.active)
        } else {
            false // Pod destroyed
        };

        // If action is no longer SatisfyRest, or pod lost power, they wake up
        if action.current != ActionType::SatisfyRest || !pod_active {
            commands.entity(pop_entity).remove::<SleepingInHypnoPod>();
            commands.entity(pop_entity).insert(MentalFog {
                duration: 1000.0, // 1000 ticks of fog
                movement_penalty: 0.5,
                work_speed_penalty: 0.5,
            });
        }
    }
}

/// Decays the MentalFog component and removes it when it expires.
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
