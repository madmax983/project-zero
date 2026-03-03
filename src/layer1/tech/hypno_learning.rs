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

#[derive(Component, Debug, Clone, Default)]
pub struct MentalFog {
    pub duration: f32,           // Ticks
    pub movement_penalty: f32,   // Multiplier (e.g. 0.5)
    pub work_speed_penalty: f32, // Multiplier (e.g. 0.5)
}

#[derive(Component, Debug, Clone)]
pub struct SleepingInHypnoPod {
    pub bed_entity: Entity,
}

#[allow(clippy::type_complexity)]
pub fn apply_mental_fog_modifiers_system(
    mut query: Query<(&MentalFog, Option<&mut crate::layer1::pop::Speed>)>,
) {
    for (fog, speed_opt) in query.iter_mut() {
        if let Some(mut speed) = speed_opt {
            speed.current *= fog.movement_penalty;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn hypno_sleep_system(
    mut commands: Commands,
    mut pops: Query<
        (
            Entity,
            &mut crate::layer1::skills::Skills,
            &mut crate::layer1::needs::Needs,
            &crate::layer1::AssignedTo,
            &crate::layer1::utility_types::PopAction,
            &crate::layer1::lifecycle::Age,
            Option<&mut crate::layer1::traits::Traits>,
        ),
        With<crate::layer1::pop::Pop>,
    >,
    pods: Query<(&HypnoPod, Option<&crate::layer1::energy::PowerConsumer>)>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    for (entity, mut skills, mut needs, assignment, action, age, mut traits_opt) in pops.iter_mut()
    {
        if action.current == crate::layer1::utility_types::ActionType::SatisfyRest {
            if let Ok((pod, power_opt)) = pods.get(assignment.entity) {
                // Must be powered if it has a PowerConsumer
                let is_powered = power_opt.is_none_or(|p| p.active);

                if !is_powered {
                    continue;
                }

                // Marker to ensure wake_up detection
                commands.entity(entity).insert(SleepingInHypnoPod {
                    bed_entity: assignment.entity,
                });

                // Child check: irreversible brain damage (trauma) instead of XP
                let is_adult = age.ticks_alive >= 18 * crate::layer1::balance::TICKS_PER_YEAR;
                if !is_adult {
                    if let Some(ref mut traits) = traits_opt {
                        traits.0.insert(crate::layer1::traits::Trait::Volatile);
                    } else {
                        let mut traits_set = std::collections::HashSet::new();
                        traits_set.insert(crate::layer1::traits::Trait::Volatile);
                        commands
                            .entity(entity)
                            .insert(crate::layer1::traits::Traits(traits_set));
                    }
                    if let Some(ref mut l) = log {
                        l.add("A child suffered brain damage from a Hypno-Pod!".to_string());
                    }
                    continue; // No XP or extra hunger
                }

                // Grant XP
                skills.add_xp(pod.target_skill, pod.xp_rate);

                // Drain Hunger extra
                needs.hunger = (needs.hunger - 0.002).max(0.0); // Extra drain rate
            }
        }
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    mut pops: Query<
        (
            Entity,
            &SleepingInHypnoPod,
            &crate::layer1::utility_types::PopAction,
        ),
        With<crate::layer1::pop::Pop>,
    >,
    pods: Query<&HypnoPod>,
) {
    for (entity, sleeping_marker, action) in pops.iter_mut() {
        // If they are no longer resting, they have woken up
        if action.current != crate::layer1::utility_types::ActionType::SatisfyRest {
            if pods.get(sleeping_marker.bed_entity).is_ok() {
                // Apply Fog
                commands.entity(entity).insert(MentalFog {
                    duration: 1000.0,      // Long duration
                    movement_penalty: 0.5, // Half speed
                    work_speed_penalty: 0.5,
                });
            }
            // Always remove the marker once they stop resting
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
