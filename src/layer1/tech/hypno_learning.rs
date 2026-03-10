use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;
use crate::layer1::skills::{SkillType, XpGainEvent, XpSource};
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::utility_types::{ActionType, PopAction};
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
            xp_rate: 10.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct MentalFog {
    pub duration: f32,
    pub movement_penalty: f32,
    pub work_speed_penalty: f32,
}

#[derive(Component, Debug, Clone)]
pub struct HypnoSleepTracker;

pub fn hypno_sleep_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Needs, &PopAction, &AssignedTo), With<Pop>>,
    pods: Query<(&HypnoPod, Option<&PowerConsumer>)>,
    mut xp_events: EventWriter<XpGainEvent>,
) {
    for (entity, mut needs, action, assigned) in pops.iter_mut() {
        if action.current == ActionType::SatisfyRest && assigned.assignment_type == AssignmentType::HousingResident {
            if let Ok((pod, power_opt)) = pods.get(assigned.entity) {
                let has_power = power_opt.map_or(true, |p| p.active);
                if has_power {
                    xp_events.send(XpGainEvent {
                        entity,
                        skill: pod.target_skill,
                        amount: pod.xp_rate,
                        source: XpSource::Action,
                    });
                }
                needs.hunger = (needs.hunger - 0.01).max(0.0);
                commands.entity(entity).insert(HypnoSleepTracker);
            }
        }
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    pops: Query<(Entity, &PopAction), With<HypnoSleepTracker>>,
) {
    for (entity, action) in pops.iter() {
        if action.current != ActionType::SatisfyRest {
            commands.entity(entity).remove::<HypnoSleepTracker>();
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
    mut fogs: Query<(Entity, &mut MentalFog)>,
) {
    for (entity, mut fog) in fogs.iter_mut() {
        fog.duration -= 1.0;
        if fog.duration <= 0.0 {
            commands.entity(entity).remove::<MentalFog>();
        }
    }
}
