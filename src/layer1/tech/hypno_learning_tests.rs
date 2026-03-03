#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::balance::TICKS_PER_YEAR;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::execution::components::{AtTarget, MovementTarget};
    use crate::layer1::housing::Housing;
    use crate::layer1::lifecycle::Age;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::utility_types::{ActionType, PopAction};
    use crate::layer1::tech::hypno_learning::{
        hypno_sleep_system, update_mental_fog_system, wake_up_hypno_system,
        HypnoPod, MentalFog, SleepingInHypnoPod,
    };
    use crate::layer1::map::GridPosition;
    use bevy_ecs::system::RunSystemOnce;

    fn setup() -> World {
        World::new()
    }

    #[test]
    fn test_hypno_pod_grants_xp_while_sleeping() {
        let mut world = setup();

        let pod = world.spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            Housing::default(),
            PowerConsumer {
                demand: 15.0,
                active: true,
            },
        )).id();

        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs::default(),
            Traits::default(),
            Age { ticks_alive: 20 * TICKS_PER_YEAR, stage: crate::layer1::lifecycle::LifeStage::Adult },
            PopAction { current: ActionType::SatisfyRest, ..Default::default() },
            MovementTarget {
                target_entity: pod,
                target_position: GridPosition::default(),
                for_action: ActionType::SatisfyRest,
            },
            AtTarget,
        )).id();

        world.run_system_once(hypno_sleep_system).unwrap();

        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert!((mining_xp - 10.0).abs() < f32::EPSILON);

        // Assert marker was added
        assert!(world.get::<SleepingInHypnoPod>(pop).is_some());
    }

    #[test]
    fn test_hypno_sleep_drains_hunger_faster() {
        let mut world = setup();

        let pod = world.spawn((
            HypnoPod::default(),
            Housing::default(),
            PowerConsumer {
                demand: 15.0,
                active: true,
            },
        )).id();

        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs { hunger: 0.8, ..Default::default() },
            Traits::default(),
            Age { ticks_alive: 20 * TICKS_PER_YEAR, stage: crate::layer1::lifecycle::LifeStage::Adult },
            PopAction { current: ActionType::SatisfyRest, ..Default::default() },
            MovementTarget {
                target_entity: pod,
                target_position: GridPosition::default(),
                for_action: ActionType::SatisfyRest,
            },
            AtTarget,
        )).id();

        world.run_system_once(hypno_sleep_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!((needs.hunger - 0.795).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hypno_sleep_traumatizes_children() {
        let mut world = setup();

        let pod = world.spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            Housing::default(),
            PowerConsumer {
                demand: 15.0,
                active: true,
            },
        )).id();

        let pop = world.spawn((
            Pop,
            Skills::default(),
            Needs::default(),
            Traits::default(),
            Age { ticks_alive: 10 * TICKS_PER_YEAR, stage: crate::layer1::lifecycle::LifeStage::Child }, // Child!
            PopAction { current: ActionType::SatisfyRest, ..Default::default() },
            MovementTarget {
                target_entity: pod,
                target_position: GridPosition::default(),
                for_action: ActionType::SatisfyRest,
            },
            AtTarget,
        )).id();

        world.run_system_once(hypno_sleep_system).unwrap();

        // No XP gained
        let skills = world.get::<Skills>(pop).unwrap();
        assert!((skills.get_xp(SkillType::Mining) - 0.0).abs() < f32::EPSILON);

        // Traumatized trait applied
        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::Traumatized));
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = setup();

        let pod = world.spawn((
            HypnoPod::default(),
            PowerConsumer {
                demand: 15.0,
                active: true,
            },
        )).id();

        let pop = world.spawn((
            Pop,
            SleepingInHypnoPod(pod),
            PopAction { current: ActionType::Idle, ..Default::default() }, // No longer resting
        )).id();

        world.run_system_once(wake_up_hypno_system).unwrap();

        assert!(world.get::<SleepingInHypnoPod>(pop).is_none());

        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        let fog = fog.unwrap();
        assert!((fog.duration - 1000.0).abs() < f32::EPSILON);
        assert!((fog.movement_penalty - 0.5).abs() < f32::EPSILON);
        assert!((fog.work_speed_penalty - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_power_loss_causes_wake_up() {
        let mut world = setup();

        let pod = world.spawn((
            HypnoPod::default(),
            PowerConsumer {
                demand: 15.0,
                active: false, // POWER OUT
            },
        )).id();

        let pop = world.spawn((
            Pop,
            SleepingInHypnoPod(pod),
            PopAction { current: ActionType::SatisfyRest, ..Default::default() }, // Still trying to rest
        )).id();

        world.run_system_once(wake_up_hypno_system).unwrap();

        // Woke up due to power loss
        assert!(world.get::<SleepingInHypnoPod>(pop).is_none());
        assert!(world.get::<MentalFog>(pop).is_some());
    }

    #[test]
    fn test_update_mental_fog_system_decays_and_removes() {
        let mut world = setup();

        let pop = world.spawn((
            MentalFog {
                duration: 1.0,
                movement_penalty: 0.5,
                work_speed_penalty: 0.5,
            },
        )).id();

        world.run_system_once(update_mental_fog_system).unwrap();

        // Duration was 1.0, decayed by 1.0, now 0.0 -> Removed
        assert!(world.get::<MentalFog>(pop).is_none());
    }
}
