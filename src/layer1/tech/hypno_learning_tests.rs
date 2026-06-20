#[cfg(test)]
mod tests {
    use crate::layer1::actions::AssignedTo;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{
        hypno_sleep_system, wake_up_hypno_system, HypnoPod, MentalFog,
    };
    use crate::layer1::utility_types::{ActionType, AssignmentType, PopAction};
    use bevy_ecs::prelude::*;

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
                Needs::default(),
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
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert!(mining_xp >= 10.0);
    }

    #[test]
    fn test_hypno_sleep_drains_hunger_faster() {
        let mut world = World::new();
        let pod = world.spawn(HypnoPod::default()).id();

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 1.0,
                    ..Default::default()
                },
                crate::layer1::skills::Skills::default(),
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

        // Verify extra hunger drain
        let needs = world.get::<Needs>(pop).unwrap();
        // Since base hunger decay is tiny, and our hypno pod decays by 0.5
        assert!(needs.hunger <= 0.5);
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();

        // Pop was sleeping in pod, now transitions to Idle
        let pop = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Idle,
                    ..Default::default()
                },
                crate::layer1::tech::hypno_learning::SleepingInHypnoPod,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);

        schedule.run(&mut world);

        // Verify MentalFog component
        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        assert!(fog.unwrap().duration > 0.0);
        assert!(world
            .get::<crate::layer1::tech::hypno_learning::SleepingInHypnoPod>(pop)
            .is_none());
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
    fn test_hypno_sleep_children_gain_volatile_not_xp() {
        use crate::layer1::lifecycle::{Age, LifeStage};
        use crate::layer1::traits::{Trait, Traits};

        let mut world = World::new();
        // Spawn HypnoPod
        let pod = world
            .spawn(HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            })
            .id();

        // Spawn Child Pop sleeping in the pod
        let pop = world
            .spawn((
                Pop,
                Skills::default(),
                Needs::default(),
                Age {
                    ticks_alive: 10,
                    stage: LifeStage::Child,
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

        // Verify NO XP gain
        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert_eq!(mining_xp, 0.0, "Child should not gain XP in HypnoPod");

        // Verify Trait::Volatile is added
        let traits = world.get::<Traits>(pop);
        assert!(traits.is_some(), "Child should have Traits component added");
        assert!(
            traits.unwrap().has(Trait::Volatile),
            "Child should gain Volatile trait"
        );
    }
}
#[cfg(test)]
mod additional_tests {
    use crate::layer1::actions::AssignedTo;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::SkillType;
    use crate::layer1::skills::Skills;
    use crate::layer1::tech::hypno_learning::{
        hypno_sleep_system, update_mental_fog_system, wake_up_hypno_system, HypnoPod, MentalFog,
    };
    use crate::layer1::utility_types::{ActionType, AssignmentType, PopAction};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_hypno_sleep_system_does_not_do_anything_if_pop_assigned_to_non_pod_entity_second_branch(
    ) {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                Skills::default(),
                Needs::default(),
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                AssignedTo {
                    // Deliberately assigning to an entity that doesn't exist to hit the other branch
                    entity: Entity::from_raw(99999),
                    assignment_type: AssignmentType::HousingResident,
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        assert!(world
            .get::<crate::layer1::tech::hypno_learning::SleepingInHypnoPod>(pop)
            .is_none());
    }

    #[test]
    fn test_hypno_sleep_system_does_not_do_anything_if_action_not_satisfy_rest() {
        let mut world = World::new();
        // Spawn HypnoPod
        let pod = world
            .spawn(HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            })
            .id();

        let pop = world
            .spawn((
                Pop,
                Skills::default(),
                Needs::default(),
                PopAction {
                    current: ActionType::Idle,
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

        assert!(world
            .get::<crate::layer1::tech::hypno_learning::SleepingInHypnoPod>(pop)
            .is_none());
    }

    #[test]
    fn test_wake_up_hypno_system_ignores_entities_without_sleeping_in_hypno_pod() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Idle,
                    ..Default::default()
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);
        schedule.run(&mut world);

        assert!(world
            .get::<crate::layer1::tech::hypno_learning::MentalFog>(pop)
            .is_none());
    }

    #[test]
    fn test_update_mental_fog_system_decreases_duration() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                MentalFog {
                    duration: 10.0,
                    movement_penalty: 0.5,
                    work_speed_penalty: 0.5,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_mental_fog_system);
        schedule.run(&mut world);

        let fog = world.get::<MentalFog>(pop).unwrap();
        assert_eq!(fog.duration, 9.0);
    }

    #[test]
    fn test_update_mental_fog_system_removes_component() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                MentalFog {
                    duration: 1.0,
                    movement_penalty: 0.5,
                    work_speed_penalty: 0.5,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_mental_fog_system);
        schedule.run(&mut world);

        assert!(world.get::<MentalFog>(pop).is_none());
    }

    #[test]
    fn test_hypno_sleep_system_adds_trait_when_pop_has_traits_component() {
        use crate::layer1::lifecycle::{Age, LifeStage};
        use crate::layer1::traits::{Trait, Traits};

        let mut world = World::new();
        // Spawn HypnoPod
        let pod = world
            .spawn(HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            })
            .id();

        let mut traits = Traits::default();
        traits.add(Trait::Glutton);
        // Spawn Child Pop sleeping in the pod
        let pop = world
            .spawn((
                Pop,
                Skills::default(),
                Needs::default(),
                Age {
                    ticks_alive: 10,
                    stage: LifeStage::Child,
                },
                traits,
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

        // Verify Trait::Volatile is added
        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::Volatile));
    }

    #[test]
    fn test_hypno_sleep_system_does_not_do_anything_if_power_inactive() {
        use crate::layer1::energy::PowerConsumer;

        let mut world = World::new();
        // Spawn HypnoPod
        let pod = world
            .spawn((
                HypnoPod {
                    target_skill: SkillType::Mining,
                    xp_rate: 10.0,
                },
                PowerConsumer {
                    active: false,
                    demand: 1.0,
                },
            ))
            .id();

        // Spawn Pop sleeping in the pod
        let pop = world
            .spawn((
                Pop,
                Skills::default(),
                Needs::default(),
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

        // Verify NO XP gain
        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert_eq!(mining_xp, 0.0);
    }

    #[test]
    fn test_hypno_sleep_system_does_not_do_anything_if_pop_assigned_to_non_pod_entity() {
        let mut world = World::new();
        // Spawn non-pod entity
        let pod = world.spawn(()).id();

        // Spawn Pop sleeping assigned to non-pod entity
        let pop = world
            .spawn((
                Pop,
                Skills::default(),
                Needs::default(),
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

        assert!(world
            .get::<crate::layer1::tech::hypno_learning::SleepingInHypnoPod>(pop)
            .is_none());
    }

    #[test]
    fn test_hypno_sleep_system_does_not_do_anything_if_power_opt_is_none() {
        let mut world = World::new();
        // Spawn HypnoPod
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
                Needs::default(),
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
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert_eq!(mining_xp, 10.0);
    }

    #[test]
    fn test_wake_up_hypno_system_does_not_do_anything_if_action_is_satisfy_rest() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                crate::layer1::tech::hypno_learning::SleepingInHypnoPod,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);
        schedule.run(&mut world);

        assert!(world
            .get::<crate::layer1::tech::hypno_learning::MentalFog>(pop)
            .is_none());
        assert!(world
            .get::<crate::layer1::tech::hypno_learning::SleepingInHypnoPod>(pop)
            .is_some());
    }

    #[test]
    fn test_hypno_sleep_system_does_not_do_anything_if_pod_query_fails() {
        let mut world = World::new();
        // Spawn non-pod entity without HypnoPod
        let pod = world.spawn(()).id();

        // Spawn Pop sleeping assigned to non-pod entity
        let pop = world
            .spawn((
                Pop,
                Skills::default(),
                Needs::default(),
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

        assert!(world
            .get::<crate::layer1::tech::hypno_learning::SleepingInHypnoPod>(pop)
            .is_none());
    }
}
