use bevy_ecs::prelude::*;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::pop::{Pop};
use crate::layer1::needs::Needs;
use crate::layer1::utility_types::{PopAction, ActionType};
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::energy::PowerConsumer;

#[derive(Component, Debug, Clone)]
pub struct HypnoPod {
    pub target_skill: SkillType,
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
    pub duration: f32, // Ticks
    pub movement_penalty: f32, // 0.0 to 1.0 (multiplier)
    pub work_speed_penalty: f32,
}

#[derive(Event, Debug, Clone)]
pub struct WakeUpHypnoEvent {
    pub entity: Entity,
    pub bed_entity: Entity,
}

pub fn hypno_sleep_system(
    mut pops: Query<(Entity, &mut Skills, &mut Needs, &PopAction, &AssignedTo), With<Pop>>,
    pods: Query<(&HypnoPod, Option<&PowerConsumer>)>,
) {
    for (_, mut skills, mut needs, action, assigned) in pops.iter_mut() {
        if action.current == ActionType::SatisfyRest && assigned.assignment_type == AssignmentType::HousingResident {
            if let Ok((pod, power_opt)) = pods.get(assigned.entity) {
                // Check if powered
                if let Some(power) = power_opt {
                    if !power.active {
                        continue; // No power, no learning
                    }
                }

                // Grant XP
                skills.add_xp(pod.target_skill, pod.xp_rate);

                // Drain Hunger extra
                needs.hunger = (needs.hunger - 0.05).max(0.0); // Extra brain calorie drain
            }
        }
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    mut events: EventReader<WakeUpHypnoEvent>,
    pods: Query<(&HypnoPod, Option<&PowerConsumer>)>,
) {
    for event in events.read() {
        if let Ok((_, power_opt)) = pods.get(event.bed_entity) {
            let has_power = power_opt.map_or(true, |p| p.active);

            // Severe fog if power cut, moderate if normal
            let duration = if has_power { 1000.0 } else { 2000.0 };
            let penalty = if has_power { 0.5 } else { 0.2 };

            // Apply Fog
            commands.entity(event.entity).insert(MentalFog {
                duration,
                movement_penalty: penalty,
                work_speed_penalty: penalty,
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

pub fn apply_mental_fog_penalties_system(
    mut query: Query<(&mut crate::layer1::pop::Speed, &MentalFog)>
) {
    for (mut speed, fog) in query.iter_mut() {
        speed.current *= fog.movement_penalty;
    }
}
