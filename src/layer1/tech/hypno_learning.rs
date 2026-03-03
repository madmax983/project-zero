use bevy_ecs::prelude::*;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::needs::Needs;
use crate::layer1::actions::AssignmentType;
use crate::layer1::actions::AssignedTo;
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
            xp_rate: 0.1,
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct MentalFog {
    pub duration: f32,
    pub movement_penalty: f32,
    pub work_speed_penalty: f32,
}

pub fn hypno_sleep_system(
    mut pops: Query<(&mut Skills, &mut Needs, &AssignedTo)>,
    pods: Query<(&HypnoPod, Option<&PowerConsumer>)>,
) {
    for (mut skills, mut needs, assigned) in pops.iter_mut() {
        if assigned.assignment_type == AssignmentType::HousingResident {
            if let Ok((pod, power_opt)) = pods.get(assigned.entity) {
                // Power check
                let has_power = power_opt.map_or(true, |p| p.active);

                if has_power {
                    let skill_type: SkillType = pod.target_skill.clone();
                    let xp_rate: f32 = pod.xp_rate;
                    Skills::add_xp(&mut skills, skill_type, xp_rate);
                }

                needs.hunger = (needs.hunger - 0.5).max(0.0);
            }
        }
    }
}

pub mod events {
    use bevy_ecs::prelude::*;
    #[derive(Event)]
    pub struct WakeUpEvent {
        pub entity: Entity,
        pub bed_entity: Option<Entity>,
    }
}

pub fn wake_up_hypno_system(
    mut commands: Commands,
    mut events: EventReader<events::WakeUpEvent>,
    pods: Query<(&HypnoPod, Option<&PowerConsumer>)>,
) {
    for event in events.read() {
        if let Some(bed) = event.bed_entity {
            if let Ok((_, power_opt)) = pods.get(bed) {
                let has_power = power_opt.map_or(true, |p| p.active);
                let duration = if has_power { 1000.0 } else { 2000.0 }; // Worse if power fails

                commands.entity(event.entity).insert(MentalFog {
                    duration,
                    movement_penalty: 0.5,
                    work_speed_penalty: 0.5,
                });
            }
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
    mut query: Query<(&MentalFog, &mut crate::layer1::pop::Speed)>,
) {
    for (fog, mut speed) in query.iter_mut() {
        speed.current *= fog.movement_penalty;
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{HypnoPod, MentalFog, events::WakeUpEvent, apply_mental_fog_penalties_system};
    use crate::layer1::actions::AssignmentType;
    use crate::layer1::actions::AssignedTo;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::pop::Speed;

    #[test]
    fn test_hypno_pod_grants_xp_while_sleeping() {
        let mut world = World::new();
        let pod = world.spawn((HypnoPod {
            target_skill: SkillType::Mining,
            xp_rate: 10.0,
        }, PowerConsumer { active: true, demand: 10.0 })).id();

        let pop = world.spawn((
            Pop,
            Skills::default(),
            AssignedTo { entity: pod, assignment_type: AssignmentType::HousingResident },
            Needs::default(), // To avoid query failing
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(super::hypno_sleep_system);
        schedule.run(&mut world);

        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert!(mining_xp >= 10.0);
    }

    #[test]
    fn test_hypno_sleep_drains_hunger_faster() {
        let mut world = World::new();
        let pod = world.spawn((HypnoPod::default(), PowerConsumer { active: true, demand: 10.0 })).id();

        let pop = world.spawn((
            Pop,
            Skills::default(), // Needs Skills otherwise Query won't match
            Needs { hunger: 100.0, rest: 100.0, leisure: 100.0, hygiene: 100.0 },
            AssignedTo { entity: pod, assignment_type: AssignmentType::HousingResident },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(super::hypno_sleep_system);
        schedule.run(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger < 100.0);
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();
        let pod = world.spawn((HypnoPod::default(), PowerConsumer { active: true, demand: 10.0 })).id();

        // manually init resource since Default trait on Events<WakeUpEvent> fails without Default on Entity
        world.insert_resource(Events::<WakeUpEvent>::default());

        let pop = world.spawn((
            Pop,
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(super::wake_up_hypno_system);

        world.send_event(WakeUpEvent {
            entity: pop,
            bed_entity: Some(pod),
        });

        schedule.run(&mut world);

        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        assert!(fog.unwrap().duration > 0.0);
    }

    #[test]
    fn test_mental_fog_penalizes_movement_and_work() {
        let fog = MentalFog {
            duration: 10.0,
            movement_penalty: 0.5,
            work_speed_penalty: 0.5,
        };
        assert_eq!(fog.movement_penalty, 0.5);
    }

    #[test]
    fn test_power_failure_severe_fog() {
        let mut world = World::new();
        let pod = world.spawn((HypnoPod::default(), PowerConsumer { active: false, demand: 10.0 })).id();
        world.insert_resource(Events::<WakeUpEvent>::default());
        let pop = world.spawn(Pop).id();
        let mut schedule = Schedule::default();
        schedule.add_systems(super::wake_up_hypno_system);
        world.send_event(WakeUpEvent {
            entity: pop,
            bed_entity: Some(pod),
        });
        schedule.run(&mut world);

        let fog = world.get::<MentalFog>(pop).unwrap();
        assert!(fog.duration > 1500.0); // Severe fog
    }

    #[test]
    fn test_speed_penalty() {
        let mut world = World::new();
        let pop = world.spawn((MentalFog { duration: 10.0, movement_penalty: 0.5, work_speed_penalty: 0.5 }, Speed { current: 1.0, ..Default::default() })).id();
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_mental_fog_penalties_system);
        schedule.run(&mut world);
        let speed = world.get::<Speed>(pop).unwrap();
        assert_eq!(speed.current, 0.5);
    }
}
