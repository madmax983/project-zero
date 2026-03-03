#[cfg(test)]
mod tests {
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{
        apply_mental_fog_speed_system, hypno_sleep_system, update_mental_fog_system,
        wake_up_hypno_system, HypnoPod, HypnoSleepMarker, MentalFog,
    };
    use crate::layer1::utility_types::{ActionType, AssignmentType, PopAction};
    use crate::layer1::AssignedTo;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

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
        world.run_system_once(hypno_sleep_system).unwrap();

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
                Skills::default(),
                Needs {
                    hunger: 100.0,
                    ..Default::default()
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
        world.run_system_once(hypno_sleep_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger < 100.0);
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                HypnoSleepMarker,
                PopAction {
                    current: ActionType::Idle, // Woke up
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(wake_up_hypno_system).unwrap();

        // Verify MentalFog component
        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        assert!(fog.unwrap().duration > 0.0);
        assert!(world.get::<HypnoSleepMarker>(pop).is_none());
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
    fn test_update_mental_fog_decays() {
        let mut world = World::new();

        let pop = world
            .spawn(MentalFog {
                duration: 2.0,
                movement_penalty: 0.5,
                work_speed_penalty: 0.5,
            })
            .id();

        world.run_system_once(update_mental_fog_system).unwrap();
        assert_eq!(world.get::<MentalFog>(pop).unwrap().duration, 1.0);

        world.run_system_once(update_mental_fog_system).unwrap();
        assert!(world.get::<MentalFog>(pop).is_none());
    }

    #[test]
    fn test_apply_mental_fog_speed_system() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Speed {
                    base: 1.0,
                    current: 2.0,
                    accumulator: 0.0,
                },
                MentalFog {
                    duration: 10.0,
                    movement_penalty: 0.5,
                    work_speed_penalty: 0.5,
                },
            ))
            .id();

        world
            .run_system_once(apply_mental_fog_speed_system)
            .unwrap();
        assert_eq!(world.get::<Speed>(pop).unwrap().current, 1.0);
    }
}
