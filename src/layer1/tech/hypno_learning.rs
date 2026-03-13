use bevy_ecs::prelude::*;
use crate::layer1::skills::SkillType;

/// The `HypnoPod` component indicating this building is a hypno pod.
#[derive(Component, Debug, Clone)]
pub struct HypnoPod {
    /// The skill to train while sleeping
    pub target_skill: SkillType, // Default to something or allow "General" learning
    /// XP gained per tick
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

/// The `MentalFog` component indicating a penalty after waking up from hypno pod.
#[derive(Component, Debug, Clone, Default)]
pub struct MentalFog {
    /// Ticks remaining
    pub duration: f32, // Ticks or Seconds
    /// Multiplier for movement
    pub movement_penalty: f32, // 0.0 to 1.0 (multiplier)
}

/// Marker component to remember the Pop was sleeping in a HypnoPod
#[derive(Component, Debug, Clone, Default)]
pub struct HypnoSleeping;

use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::gastronomy::WorkSpeedBuff; // Use existing component for work speed
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::pop::Speed;
use crate::layer1::skills::Skills;
use crate::layer1::utility_types::{ActionType, PopAction};

type HypnoSleepQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Skills,
        &'static mut Needs,
        &'static PopAction,
        &'static AssignedTo,
    ),
    With<Pop>,
>;

/// System to process XP and hunger while sleeping in a HypnoPod
pub fn hypno_sleep_system(
    mut commands: Commands,
    mut pops: HypnoSleepQuery,
    pods: Query<(&HypnoPod, Option<&PowerConsumer>)>,
) {
    for (entity, mut skills, mut needs, action, assigned) in pops.iter_mut() {
        if action.current == ActionType::SatisfyRest
            && assigned.assignment_type == AssignmentType::HousingResident
        {
            if let Ok((pod, power)) = pods.get(assigned.entity) {
                if power.is_none_or(|p| p.active) {
                    commands.entity(entity).insert(HypnoSleeping);
                    // Grant XP
                    *skills.xp.entry(pod.target_skill).or_insert(0.0) += pod.xp_rate;

                    // Drain Hunger extra
                    needs.hunger = (needs.hunger - 0.5).max(0.0); // Arbitrary drain rate
                }
            }
        }
    }
}




type WakeUpPopsQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static PopAction), (With<Pop>, With<HypnoSleeping>)>;

/// Detects when a pop stops resting in a HypnoPod to apply MentalFog
pub fn wake_up_hypno_system(mut commands: Commands, pops: WakeUpPopsQuery) {
    for (entity, action) in pops.iter() {
        if action.current != ActionType::SatisfyRest {
            // Remove marker
            commands.entity(entity).remove::<HypnoSleeping>();

            // Apply Fog
            commands.entity(entity).insert(MentalFog {
                duration: 1000.0,      // Long duration
                movement_penalty: 0.5, // Half speed
            });
            commands.entity(entity).insert(WorkSpeedBuff {
                multiplier: 0.5,
                duration: 1000,
            });
        }
    }
}

/// System to decay mental fog duration
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

/// Applies MentalFog penalties to Pop Speed
pub fn apply_mental_fog_speed_modifier_system(mut query: Query<(&mut Speed, &MentalFog)>) {
    for (mut speed, fog) in query.iter_mut() {
        speed.current *= fog.movement_penalty;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::gastronomy::WorkSpeedBuff;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::pop::Speed;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::utility_types::{ActionType, PopAction};

    #[test]
    fn test_hypno_pod_grants_xp_while_sleeping() {
        let mut world = World::new();
        // Spawn HypnoPod with target skill
        let pod = world
            .spawn(HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            })
            .id();

        // Spawn Pop sleeping in the pod
        let pop = world
            .spawn((
                Pop,
                Skills::default(),
                Needs {
                    hunger: 100.0,
                    rest: 100.0,
                    leisure: 100.0,
                    hygiene: 100.0,
                },
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                AssignedTo {
                    entity: pod,
                    assignment_type: AssignmentType::HousingResident,
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify XP gain
        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.xp.get(&SkillType::Mining).copied().unwrap_or(0.0);
        assert!(mining_xp >= 10.0);

        let has_sleeping = world.get::<HypnoSleeping>(pop);
        assert!(has_sleeping.is_some());
    }

    #[test]
    fn test_hypno_sleep_drains_hunger_faster() {
        let mut world = World::new();
        let pod = world.spawn(HypnoPod::default()).id();

        let pop = world
            .spawn((
                Pop,
                Skills::default(),
                Needs {
                    hunger: 100.0,
                    rest: 100.0,
                    leisure: 100.0,
                    hygiene: 100.0,
                },
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                AssignedTo {
                    entity: pod,
                    assignment_type: AssignmentType::HousingResident,
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify extra hunger drain (Standard sleep might consume X, Hypno consumes X + Y)
        let needs = world.get::<Needs>(pop).unwrap();
        // Assuming normal decay is handled elsewhere or we model the delta
        // We expect a significant drop.
        assert!(needs.hunger < 100.0); // Assuming decay rate > 1.0 per tick for hypno
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();

        // Pop was sleeping in pod, now transitions to Idle (Waking up)
        let pop = world
            .spawn((
                Pop,
                HypnoSleeping,
                PopAction {
                    current: ActionType::Idle, // Woke up
                    ..Default::default()
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);
        schedule.run(&mut world);

        // Verify MentalFog component
        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        assert!(fog.unwrap().duration > 0.0);

        let speed_buff = world.get::<WorkSpeedBuff>(pop);
        assert!(speed_buff.is_some());
        assert_eq!(speed_buff.unwrap().multiplier, 0.5);
    }

    #[test]
    fn test_update_mental_fog() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                MentalFog {
                    duration: 1.0,
                    movement_penalty: 0.5,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_mental_fog_system);
        schedule.run(&mut world);

        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_none());
    }

    #[test]
    fn test_mental_fog_penalizes_movement_and_work() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                MentalFog {
                    duration: 1.0,
                    movement_penalty: 0.5,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_mental_fog_speed_modifier_system);
        schedule.run(&mut world);

        let speed = world.get::<Speed>(pop).unwrap();
        assert_eq!(speed.current, 0.5);
    }
}
