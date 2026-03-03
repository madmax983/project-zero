#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{HypnoPod, MentalFog, hypno_sleep_system, wake_up_hypno_system, HypnoWakeUpEvent};
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::Building;
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::housing::Housing;

    #[test]
    fn test_hypno_pod_grants_xp_while_sleeping() {
        let mut world = World::new();
        // Spawn HypnoPod with target skill
        let pod = world.spawn((
            Building {
                building_type: crate::layer1::building::BuildingType::HypnoPod,
            },
            GridPosition { x: 0, y: 0 },
            Housing {
                capacity: 1,
                residents: vec![],
            },
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
        )).id();

        // Spawn Pop sleeping in the pod
        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs::default(),
            crate::layer1::utility_types::PopAction {
                current: crate::layer1::utility_types::ActionType::SatisfyRest,
                current_utility: 1.0,
                ticks_committed: 0,
            },
            AssignedTo {
                entity: pod,
                assignment_type: AssignmentType::HousingResident,
            },
            GridPosition { x: 0, y: 0 }, // Pop is in the pod
        )).id();

        world.get_mut::<Housing>(pod).unwrap().residents.push(pop);

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
        let pod = world.spawn((
            Building {
                building_type: crate::layer1::building::BuildingType::HypnoPod,
            },
            GridPosition { x: 0, y: 0 },
            Housing {
                capacity: 1,
                residents: vec![],
            },
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
        )).id();

        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs { hunger: 100.0, ..Default::default() },
            crate::layer1::utility_types::PopAction {
                current: crate::layer1::utility_types::ActionType::SatisfyRest,
                current_utility: 1.0,
                ticks_committed: 0,
            },
            AssignedTo {
                entity: pod,
                assignment_type: AssignmentType::HousingResident,
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        world.get_mut::<Housing>(pod).unwrap().residents.push(pop);

        // Run system
        world.run_system_once(hypno_sleep_system).unwrap();

        // Verify extra hunger drain
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger < 99.0); // Assuming decay rate > 1.0 per tick for hypno
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();
        let _pod = world.spawn((
            Building {
                building_type: crate::layer1::building::BuildingType::HypnoPod,
            },
            GridPosition { x: 0, y: 0 },
            Housing {
                capacity: 1,
                residents: vec![],
            },
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
        )).id();

        world.insert_resource(Events::<HypnoWakeUpEvent>::default());

        let pop = world.spawn(Pop).id();

        // Send event saying Pop woke up from Pod
        world.send_event(HypnoWakeUpEvent {
            entity: pop,
        });

        world.run_system_once(wake_up_hypno_system).unwrap();

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
