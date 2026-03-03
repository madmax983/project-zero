use bevy_ecs::prelude::*;
use crate::layer1::skills::{Skills, SkillType, XpGainEvent, XpSource};
use crate::layer1::needs::Needs;
use crate::layer1::utility_types::{PopAction, ActionType};
use crate::layer1::actions::AssignedTo;
use crate::layer1::energy::PowerConsumer;

#[derive(Component)]
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

#[derive(Component, Default)]
pub struct MentalFog {
    pub duration: f32,
    pub movement_penalty: f32,
    pub work_speed_penalty: f32,
}

#[derive(Component)]
pub struct SleepingInHypnoPod;

pub fn hypno_sleep_system(
    mut commands: Commands,
    mut pops: Query<(
        Entity,
        &AssignedTo,
        &PopAction,
        &mut Needs,
        Option<&mut Skills>,
        Option<&SleepingInHypnoPod>,
    )>,
    pods: Query<(&HypnoPod, &PowerConsumer)>,
    mut xp_events: EventWriter<XpGainEvent>,
) {
    for (entity, assigned, action, mut needs, mut skills_opt, sleeping) in &mut pops {
        if action.current != ActionType::SatisfyRest {
            continue;
        }

        if let Ok((pod, power)) = pods.get(assigned.entity) {
            if power.active {
                // Grant XP
                if let Some(ref mut skills) = skills_opt {
                    skills.add_xp(pod.target_skill, pod.xp_rate);
                    xp_events.send(XpGainEvent {
                        entity,
                        skill: pod.target_skill,
                        amount: pod.xp_rate,
                        source: XpSource::Action,
                    });
                }

                // Extra Hunger Drain
                needs.hunger = (needs.hunger - 0.05).max(0.0);

                if sleeping.is_none() {
                    commands.entity(entity).insert(SleepingInHypnoPod);
                }
            }
        }
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    pops: Query<(Entity, &PopAction), With<SleepingInHypnoPod>>,
) {
    for (entity, action) in &pops {
        if action.current != ActionType::SatisfyRest {
            commands.entity(entity).remove::<SleepingInHypnoPod>();
            commands.entity(entity).insert(MentalFog {
                duration: 50.0, // Base duration
                movement_penalty: 0.5,
                work_speed_penalty: 0.5,
            });
        }
    }
}
pub fn update_mental_fog_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut MentalFog)>,
) {
    for (entity, mut fog) in &mut pops {
        fog.duration -= 1.0;
        if fog.duration <= 0.0 {
            commands.entity(entity).remove::<MentalFog>();
        }
    }
}
pub fn apply_mental_fog_speed_modifiers_system(
    mut query: Query<(&MentalFog, &mut crate::layer1::pop::Speed)>,
) {
    for (fog, mut speed) in &mut query {
        speed.current *= fog.movement_penalty;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::skills::Skills;
    use crate::layer1::utility_types::{ActionType, PopAction};
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::energy::PowerConsumer;

    #[test]
    fn test_hypno_pod_grants_xp_while_sleeping() {
        let mut world = World::new();
        world.insert_resource(Events::<crate::layer1::skills::XpGainEvent>::default());

        let pod = world.spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            PowerConsumer { active: true, demand: 10.0 },
        )).id();

        let pop = world.spawn((
            Pop,
            Needs { hunger: 1.0, ..Default::default() },
            Skills::default(),
            PopAction { current: ActionType::SatisfyRest, ..Default::default() },
            AssignedTo { entity: pod, assignment_type: AssignmentType::HousingResident },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert!(mining_xp >= 10.0, "Should gain XP while sleeping in HypnoPod");
    }

    #[test]
    fn test_hypno_sleep_drains_hunger_faster() {
        let mut world = World::new();
        world.insert_resource(Events::<crate::layer1::skills::XpGainEvent>::default());

        let pod = world.spawn((
            HypnoPod { target_skill: SkillType::Mining, xp_rate: 10.0 },
            PowerConsumer { active: true, demand: 10.0 },
        )).id();

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.5, ..Default::default() },
            Skills::default(),
            PopAction { current: ActionType::SatisfyRest, ..Default::default() },
            AssignedTo { entity: pod, assignment_type: AssignmentType::HousingResident },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        // Hunger decays significantly, so it should be lower than 0.5
        assert!(needs.hunger < 0.49, "Hunger should drain faster in HypnoPod");
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            SleepingInHypnoPod,
            PopAction { current: ActionType::Idle, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);
        schedule.run(&mut world);

        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some(), "Should apply MentalFog when waking up");
        assert!(world.get::<SleepingInHypnoPod>(pop).is_none(), "Should remove SleepingInHypnoPod");

        let fog = fog.unwrap();
        assert!(fog.duration > 0.0);
        assert!(fog.movement_penalty < 1.0);
        assert!(fog.work_speed_penalty < 1.0);
    }

    #[test]
    fn test_mental_fog_decays() {
        let mut world = World::new();
        let pop = world.spawn((
            MentalFog {
                duration: 1.5,
                movement_penalty: 0.5,
                work_speed_penalty: 0.5,
            },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_mental_fog_system);

        // Tick 1: drops to 0.0, stays
        schedule.run(&mut world);
        assert!(world.get::<MentalFog>(pop).is_some());

        // Tick 2: drops < 0.0, removed
        schedule.run(&mut world);
        assert!(world.get::<MentalFog>(pop).is_none(), "MentalFog should be removed when duration <= 0");
    }
}
