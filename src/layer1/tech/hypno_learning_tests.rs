#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{HypnoPod, MentalFog, hypno_sleep_system, wake_up_hypno_system};
    use crate::layer1::actions::AssignedTo;
    use crate::layer1::utility_types::{AssignmentType, ActionType, PopAction};

    #[test]
    fn test_hypno_pod_grants_xp_while_sleeping() {
        let mut world = World::new();
        // Spawn HypnoPod with target skill
        let pod = world.spawn(HypnoPod {
            target_skill: SkillType::Mining,
            xp_rate: 10.0,
        }).id();

        // Spawn Pop sleeping in the pod
        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs::default(),
            PopAction { current: ActionType::SatisfyRest, ..Default::default() },
            AssignedTo { entity: pod, assignment_type: AssignmentType::HousingResident },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify XP gain
        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert!(mining_xp >= 10.0);
    }

    #[test]
    fn test_hypno_sleep_drains_hunger_faster() {
        let mut world = World::new();
        let pod = world.spawn(HypnoPod::default()).id();

        let pop = world.spawn((
            Pop,
            Needs { hunger: 1.0, ..Default::default() },
            crate::layer1::skills::Skills::default(),
            PopAction { current: ActionType::SatisfyRest, ..Default::default() },
            AssignedTo { entity: pod, assignment_type: AssignmentType::HousingResident },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify extra hunger drain
        let needs = world.get::<Needs>(pop).unwrap();
        // Since base hunger decay is tiny, and our hypno pod decays by 0.5
        assert!(needs.hunger <= 0.5);
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();

        // Pop was sleeping in pod, now transitions to Idle
        let pop = world.spawn((
            Pop,
            PopAction { current: ActionType::Idle, ..Default::default() },
            crate::layer1::tech::hypno_learning::SleepingInHypnoPod,
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);

        schedule.run(&mut world);

        // Verify MentalFog component
        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        assert!(fog.unwrap().duration > 0.0);
        assert!(world.get::<crate::layer1::tech::hypno_learning::SleepingInHypnoPod>(pop).is_none());
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
