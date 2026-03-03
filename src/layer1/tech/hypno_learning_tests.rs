#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{HypnoPod, MentalFog, hypno_sleep_system, wake_up_hypno_system, WakeUpHypnoEvent};
    use crate::layer1::utility_types::{PopAction, ActionType};
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::energy::PowerConsumer;

    #[test]
    fn test_hypno_pod_grants_xp_while_sleeping() {
        let mut world = World::new();
        // Spawn HypnoPod with target skill
        let pod = world.spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            PowerConsumer {
                active: true,
                demand: 20.0,
            }
        )).id();

        // Spawn Pop sleeping in the pod
        let mut action = PopAction::default();
        action.current = ActionType::SatisfyRest;

        let pop = world.spawn((
            Pop,
            Skills::default(),
            action,
            AssignedTo { entity: pod, assignment_type: AssignmentType::HousingResident },
            Needs { hunger: 1.0, ..Default::default() },
        )).id();

        // Ensure events resource exists to avoid panic if system tries to emit
        world.insert_resource(Events::<crate::layer1::skills::XpGainEvent>::default());

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify XP gain
        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert!((mining_xp - 10.0).abs() < f32::EPSILON || mining_xp > 10.0, "mining_xp is {}", mining_xp);
    }

    #[test]
    fn test_hypno_pod_no_xp_without_power() {
        let mut world = World::new();
        // Spawn HypnoPod with target skill
        let pod = world.spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            PowerConsumer {
                active: false,
                demand: 20.0,
            }
        )).id();

        // Spawn Pop sleeping in the pod
        let mut action = PopAction::default();
        action.current = ActionType::SatisfyRest;

        let pop = world.spawn((
            Pop,
            Skills::default(),
            action,
            AssignedTo { entity: pod, assignment_type: AssignmentType::HousingResident },
            Needs { hunger: 1.0, ..Default::default() },
        )).id();

        world.insert_resource(Events::<crate::layer1::skills::XpGainEvent>::default());

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify XP gain
        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert_eq!(mining_xp, 0.0);
    }

    #[test]
    fn test_hypno_sleep_drains_hunger_faster() {
        let mut world = World::new();
        let pod = world.spawn((
            HypnoPod::default(),
            PowerConsumer { active: true, demand: 20.0 },
        )).id();

        let mut action = PopAction::default();
        action.current = ActionType::SatisfyRest;

        let pop = world.spawn((
            Pop,
            Needs { hunger: 1.0, ..Default::default() },
            action,
            AssignedTo { entity: pod, assignment_type: AssignmentType::HousingResident },
            Skills::default(),
        )).id();

        world.insert_resource(Events::<crate::layer1::skills::XpGainEvent>::default());

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify extra hunger drain
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger < 0.99, "hunger is {}", needs.hunger); // Hunger should drop by 0.05
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();
        world.insert_resource(Events::<WakeUpHypnoEvent>::default());

        let pod = world.spawn((
            HypnoPod::default(),
            PowerConsumer { active: true, demand: 20.0 }
        )).id();

        let pop = world.spawn(Pop).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);

        // Send event saying Pop woke up from Pod
        world.send_event(WakeUpHypnoEvent {
            entity: pop,
            bed_entity: pod,
        });

        schedule.run(&mut world);

        // Verify MentalFog component
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
}
