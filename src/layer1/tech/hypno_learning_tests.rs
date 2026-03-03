use super::hypno_learning::*;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::layer1::AssignedTo;
use bevy_ecs::prelude::*;

#[test]
fn test_hypno_pod_grants_xp_while_sleeping() {
    let mut world = World::new();

    let pod = world
        .spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            PowerConsumer {
                demand: 50.0,
                active: true, // Powered!
            },
        ))
        .id();

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
                assignment_type: crate::layer1::utility_types::AssignmentType::HousingResident,
            },
            crate::layer1::lifecycle::Age {
                ticks_alive: 50 * crate::layer1::balance::TICKS_PER_YEAR,
                stage: crate::layer1::lifecycle::LifeStage::Adult,
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(hypno_sleep_system);
    schedule.run(&mut world);

    let skills = world.get::<Skills>(pop).unwrap();
    assert!(skills.get_xp(SkillType::Mining) >= 10.0, "Should gain XP");
}

#[test]
fn test_hypno_sleep_drains_hunger_faster() {
    let mut world = World::new();

    let pod = world
        .spawn((
            HypnoPod::default(),
            PowerConsumer {
                demand: 50.0,
                active: true, // Powered!
            },
        ))
        .id();

    let pop = world
        .spawn((
            Pop,
            Skills::default(),
            Needs {
                hunger: 1.0,
                ..Default::default()
            },
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
            AssignedTo {
                entity: pod,
                assignment_type: crate::layer1::utility_types::AssignmentType::HousingResident,
            },
            crate::layer1::lifecycle::Age {
                ticks_alive: 50 * crate::layer1::balance::TICKS_PER_YEAR,
                stage: crate::layer1::lifecycle::LifeStage::Adult,
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(hypno_sleep_system);
    schedule.run(&mut world);

    let needs = world.get::<Needs>(pop).unwrap();
    // Normal sleep doesn't drain hunger extra in that specific system, but HypnoPod should drop it down.
    assert!(needs.hunger < 1.0, "Hunger should drain faster");
}

#[test]
fn test_waking_from_hypno_applies_mental_fog() {
    let mut world = World::new();

    let pod = world.spawn(HypnoPod::default()).id();

    let pop = world
        .spawn((
            Pop,
            PopAction {
                current: ActionType::SatisfyRest, // Still sleeping
                ..Default::default()
            },
            SleepingInHypnoPod { bed_entity: pod },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(wake_up_hypno_system);
    schedule.run(&mut world);

    // Change state to waking up
    world.get_mut::<PopAction>(pop).unwrap().current = ActionType::Idle;

    schedule.run(&mut world);

    let fog = world.get::<MentalFog>(pop);
    assert!(fog.is_some(), "Waking up should apply Mental Fog");
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

    // Also test the system that applies these penalties
    let mut world = World::new();
    let pop = world
        .spawn((
            Pop,
            fog,
            crate::layer1::pop::Speed {
                base: 1.0,
                current: 1.0,
                accumulator: 0.0,
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(crate::layer1::tech::hypno_learning::apply_mental_fog_modifiers_system);
    schedule.run(&mut world);

    let speed = world.get::<crate::layer1::pop::Speed>(pop).unwrap();
    assert!(
        (speed.current - 0.5).abs() < f32::EPSILON,
        "Movement speed should be penalized"
    );

    // Work speed is handled in calculate_work_amount, no component mutation test required here.
    let work_amount = crate::layer1::execution::general_work::calculate_work_amount(
        &world,
        pop,
        crate::layer1::designation::DesignationType::Mine,
        None,
        0.5,
        1.0,
        1.0,
    );
    // Standard work amount without penalty: 10 * 1.0 (tool) * 1.0 (skill) * 1.0 * 1.0 * organic * fog
    // The test environment doesn't have the full morale systems, so morale efficiency defaults to 1.0 or similar.
    // Base amount is around ~10. With a 0.5 fog penalty, it should be ~5.
    assert!(
        work_amount < 6.0,
        "Work speed should be penalized. Amount was: {}",
        work_amount
    );
}

#[test]
fn test_hypno_pod_requires_power() {
    let mut world = World::new();

    let pod = world
        .spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            PowerConsumer {
                demand: 50.0,
                active: false, // NOT powered!
            },
        ))
        .id();

    let pop = world
        .spawn((
            Pop,
            Skills::default(),
            Needs {
                hunger: 1.0,
                ..Default::default()
            },
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
            AssignedTo {
                entity: pod,
                assignment_type: crate::layer1::utility_types::AssignmentType::HousingResident,
            },
            crate::layer1::lifecycle::Age {
                ticks_alive: 50 * crate::layer1::balance::TICKS_PER_YEAR,
                stage: crate::layer1::lifecycle::LifeStage::Adult,
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(hypno_sleep_system);
    schedule.run(&mut world);

    let skills = world.get::<Skills>(pop).unwrap();
    assert!(
        skills.get_xp(SkillType::Mining) == 0.0,
        "Should NOT gain XP when unpowered"
    );
}

#[test]
fn test_child_using_hypno_gets_trauma() {
    let mut world = World::new();

    let pod = world
        .spawn((
            HypnoPod {
                target_skill: SkillType::Mining,
                xp_rate: 10.0,
            },
            PowerConsumer {
                demand: 50.0,
                active: true, // Powered!
            },
        ))
        .id();

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
                assignment_type: crate::layer1::utility_types::AssignmentType::HousingResident,
            },
            crate::layer1::lifecycle::Age {
                ticks_alive: 10 * crate::layer1::balance::TICKS_PER_YEAR,
                stage: crate::layer1::lifecycle::LifeStage::Child,
            }, // Child
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(hypno_sleep_system);
    schedule.run(&mut world);

    let skills = world.get::<Skills>(pop).unwrap();
    assert!(
        skills.get_xp(SkillType::Mining) == 0.0,
        "Children should not gain XP"
    );

    // They get Trauma instead
    let traits = world.get::<crate::layer1::traits::Traits>(pop);
    assert!(traits.is_some(), "Child should gain Trauma (Trait)");
    assert!(traits
        .unwrap()
        .0
        .contains(&crate::layer1::traits::Trait::Volatile));
}
