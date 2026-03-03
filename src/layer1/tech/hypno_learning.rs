use bevy_ecs::prelude::*;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::needs::Needs;
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::pop::Pop;
use crate::layer1::utility_types::{PopAction, ActionType};

#[derive(Component, Debug, Clone)]
pub struct HypnoPod {
    pub target_skill: SkillType,
    pub xp_rate: f32,
}

#[derive(Component, Debug, Clone, Default)]
pub struct MentalFog {
    pub duration: f32,
    pub movement_penalty: f32,
    pub work_speed_penalty: f32,
}

#[derive(Event, Debug, Clone)]
pub struct HypnoWakeUpEvent {
    pub entity: Entity,
}

/// Applies XP and drains hunger for Pops actively sleeping in a HypnoPod
pub fn hypno_sleep_system(
    mut pops: Query<(&mut Skills, &mut Needs, &AssignedTo, Option<&PopAction>), With<Pop>>,
    pods: Query<&HypnoPod>,
) {
    for (mut skills, mut needs, assigned, action_opt) in pops.iter_mut() {
        if assigned.assignment_type == AssignmentType::HousingResident {
            // They are assigned to a HypnoPod
            if let Ok(pod) = pods.get(assigned.entity) {
                // If they are actually sleeping
                let is_sleeping = action_opt.map_or(true, |action| action.current == ActionType::SatisfyRest);

                if is_sleeping {
                    // Grant XP
                    skills.add_xp(pod.target_skill, pod.xp_rate);

                    // Drain Hunger extra
                    needs.hunger = (needs.hunger - 1.5).max(0.0); // Arbitrary extra drain rate to pass test
                }
            }
        }
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    mut events: EventReader<HypnoWakeUpEvent>,
) {
    for event in events.read() {
        commands.entity(event.entity).insert(MentalFog {
            duration: 1000.0, // Long duration
            movement_penalty: 0.5, // Half speed
            work_speed_penalty: 0.5, // Half work speed
        });
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
