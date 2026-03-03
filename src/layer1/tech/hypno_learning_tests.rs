#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::utility_types::{PopAction, ActionType};
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{
        HypnoPod, MentalFog, hypno_sleep_system, wake_up_hypno_system, update_mental_fog_system,
        HYPNO_SLEEP_HUNGER_DRAIN, MENTAL_FOG_DURATION, MENTAL_FOG_MOVEMENT_PENALTY, MENTAL_FOG_WORK_SPEED_PENALTY,
    };

    use crate::layer1::tech::hypno_learning::HypnoSleeping;

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
            Needs::default(), // Ensure needs is present
            Skills::default(),
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
            HypnoSleeping { bed_entity: pod },
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
            Skills::default(), // Ensure skills is present
            Needs { hunger: 100.0, ..Default::default() },
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
            HypnoSleeping { bed_entity: pod },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify extra hunger drain
        let needs = world.get::<Needs>(pop).unwrap();
        assert!((needs.hunger - (100.0 - HYPNO_SLEEP_HUNGER_DRAIN)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();
        let pod = world.spawn(HypnoPod::default()).id();

        let pop = world.spawn((
            Pop,
            HypnoSleeping { bed_entity: pod },
            PopAction {
                current: ActionType::Idle, // Woke up
                ..Default::default()
            },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);
        schedule.run(&mut world);

        // Verify MentalFog component
        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        let fog_data = fog.unwrap();
        assert_eq!(fog_data.duration, MENTAL_FOG_DURATION);
        assert_eq!(fog_data.movement_penalty, MENTAL_FOG_MOVEMENT_PENALTY);
        assert_eq!(fog_data.work_speed_penalty, MENTAL_FOG_WORK_SPEED_PENALTY);

        // Ensure HypnoSleeping marker is removed
        assert!(world.get::<HypnoSleeping>(pop).is_none());
    }

    #[test]
    fn test_mental_fog_decays() {
        let mut world = World::new();
        let pop = world.spawn(MentalFog {
            duration: 1.0,
            movement_penalty: 0.5,
            work_speed_penalty: 0.5,
        }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_mental_fog_system);

        // Run once, duration goes to 0, component removed
        schedule.run(&mut world);

        assert!(world.get::<MentalFog>(pop).is_none());
    }

    #[test]
    fn test_waking_from_invalid_hypno_pod_still_removes_marker() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            HypnoSleeping { bed_entity: Entity::from_raw(99999) },
            PopAction {
                current: ActionType::Idle, // Woke up
                ..Default::default()
            },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);
        schedule.run(&mut world);

        // Verify MentalFog component is NOT added because bed is invalid
        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_none());

        // Ensure HypnoSleeping marker is removed
        assert!(world.get::<HypnoSleeping>(pop).is_none());
    }

    #[test]
    fn test_sleeping_in_invalid_hypno_pod_does_not_panic() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs { hunger: 100.0, ..Default::default() },
            HypnoSleeping { bed_entity: Entity::from_raw(99999) },
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify no XP gain
        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert_eq!(mining_xp, 0.0);

        // Verify no hunger drain
        let needs = world.get::<Needs>(pop).unwrap();
        assert_eq!(needs.hunger, 100.0);
    }
}
