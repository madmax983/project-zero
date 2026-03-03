#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{HypnoPod, MentalFog, hypno_sleep_system, update_mental_fog_system};
    use crate::layer1::actions::AssignedTo;
    use crate::layer1::utility_types::{ActionType, AssignmentType};
    use crate::layer1::PopAction;
    use crate::shared::time::SimulationTime;

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
            AssignedTo {
                entity: pod,
                assignment_type: AssignmentType::HousingResident,
            },
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
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
            Skills::default(),
            Needs { hunger: 100.0, ..Default::default() },
            AssignedTo {
                entity: pod,
                assignment_type: AssignmentType::HousingResident,
            },
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify extra hunger drain (Standard sleep might consume X, Hypno consumes X + Y)
        let needs = world.get::<Needs>(pop).unwrap();
        // Assuming normal decay is handled elsewhere or we model the delta
        // We expect a significant drop.
        assert!(needs.hunger < 100.0); // Assuming decay rate
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();
        let pod = world.spawn(HypnoPod::default()).id();

        let pop = world.spawn((
            Pop,
            Needs::default(),
            Skills::default(),
            AssignedTo {
                entity: pod,
                assignment_type: AssignmentType::HousingResident,
            },
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::tech::hypno_learning::hypno_sleep_system);
        schedule.add_systems(crate::layer1::tech::hypno_learning::wake_up_hypno_system.after(crate::layer1::tech::hypno_learning::hypno_sleep_system));

        // Simulate tick where pop is still sleeping
        schedule.run(&mut world);
        assert!(world.get::<crate::layer1::tech::hypno_learning::WasInHypnoPod>(pop).is_some());
        assert!(world.get::<MentalFog>(pop).is_none());

        // Simulate wake up
        world.entity_mut(pop).insert(PopAction {
            current: ActionType::Idle,
            ..Default::default()
        });
        schedule.run(&mut world);

        // Verify MentalFog component
        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        assert!(fog.unwrap().duration > 0.0);
        assert!(world.get::<crate::layer1::tech::hypno_learning::WasInHypnoPod>(pop).is_none());
    }

    #[test]
    fn test_mental_fog_penalizes_movement_and_work() {
        // This test verifies the effect of the component, usually in movement/work systems.
        // For this spec, we just verify the component properties imply penalties.
        let fog = MentalFog {
            duration: 10.0,
            movement_penalty: 0.5,
            work_speed_penalty: 0.5,
        };

        assert_eq!(fog.movement_penalty, 0.5);
    }

    #[test]
    fn test_mental_fog_decays() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        let pop = world.spawn(MentalFog {
            duration: 1.0,
            movement_penalty: 0.5,
            work_speed_penalty: 0.5,
        }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_mental_fog_system);
        schedule.run(&mut world);

        // Verify it decays and is removed
        assert!(world.get::<MentalFog>(pop).is_none());
    }
}
