#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::needs::Needs;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{HypnoPod, MentalFog, SleepingInHypnoPod, hypno_sleep_system, wake_up_hypno_system, update_mental_fog_system, apply_mental_fog_speed_system};
    use crate::layer1::utility_types::{ActionType, PopAction};
    use crate::layer1::AssignedTo;
    use crate::layer1::energy::PowerConsumer;

    fn setup_world() -> World {
        let mut world = World::new();
        world
    }

    #[test]
    fn test_hypno_pod_grants_xp_while_sleeping() {
        let mut world = setup_world();

        let pod = world.spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            PowerConsumer { active: true, demand: 10.0 }
        )).id();

        let mut action = PopAction::default();
        action.current = ActionType::SatisfyRest;

        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs { hunger: 1.0, ..Default::default() },
            action,
            AssignedTo { entity: pod, assignment_type: crate::layer1::AssignmentType::HousingResident },
        )).id();

        world.run_system_once(hypno_sleep_system).unwrap();

        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert!((mining_xp - 10.0).abs() < f32::EPSILON);
        assert!(world.get::<SleepingInHypnoPod>(pop).is_some(), "Pop should have SleepingInHypnoPod component");
    }

    #[test]
    fn test_hypno_sleep_drains_hunger_faster() {
        let mut world = setup_world();
        let pod = world.spawn((HypnoPod::default(), PowerConsumer { active: true, demand: 10.0 })).id();

        let mut action = PopAction::default();
        action.current = ActionType::SatisfyRest;

        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs { hunger: 100.0, ..Default::default() },
            action,
            AssignedTo { entity: pod, assignment_type: crate::layer1::AssignmentType::HousingResident },
        )).id();

        world.run_system_once(hypno_sleep_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!((needs.hunger - 99.5).abs() < f32::EPSILON, "Hunger should drain by 0.5");
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = setup_world();
        let _pod = world.spawn(HypnoPod::default()).id();

        let mut action = PopAction::default();
        action.current = ActionType::Idle; // No longer resting

        let pop = world.spawn((
            Pop,
            action,
            SleepingInHypnoPod,
        )).id();

        world.run_system_once(wake_up_hypno_system).unwrap();

        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        assert!(world.get::<SleepingInHypnoPod>(pop).is_none());
        assert!((fog.unwrap().duration - 1000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mental_fog_penalizes_movement_and_work() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop,
            Speed { current: 1.0, base: 1.0, accumulator: 0.0 },
            MentalFog {
                duration: 10.0,
                movement_penalty: 0.5,
                work_speed_penalty: 0.5,
            }
        )).id();

        world.run_system_once(apply_mental_fog_speed_system).unwrap();

        let speed = world.get::<Speed>(pop).unwrap();
        assert!((speed.current - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mental_fog_decays() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop,
            MentalFog {
                duration: 1.0,
                movement_penalty: 0.5,
                work_speed_penalty: 0.5,
            }
        )).id();

        world.run_system_once(update_mental_fog_system).unwrap();

        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_none(), "Fog should decay completely");
    }

    #[test]
    fn test_hypno_sleep_requires_power() {
        let mut world = setup_world();

        let pod = world.spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            PowerConsumer { active: false, demand: 10.0 }
        )).id();

        let mut action = PopAction::default();
        action.current = ActionType::SatisfyRest;

        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs { hunger: 1.0, ..Default::default() },
            action,
            AssignedTo { entity: pod, assignment_type: crate::layer1::AssignmentType::HousingResident },
        )).id();

        world.run_system_once(hypno_sleep_system).unwrap();

        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert!(mining_xp == 0.0, "Should not grant XP if unpowered");
        assert!(world.get::<SleepingInHypnoPod>(pop).is_some(), "Should gain HypnoPod sleeper status even if unpowered");
    }
}
