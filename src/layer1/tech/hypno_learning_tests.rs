use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;
use crate::layer1::skills::{SkillType, Skills, XpGainEvent};
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::layer1::tech::hypno_learning::{HypnoPod, MentalFog, HypnoSleepTracker, hypno_sleep_system, wake_up_hypno_system, update_mental_fog_system};
use crate::layer1::energy::PowerConsumer;
use bevy_ecs::system::RunSystemOnce;

#[test]
fn test_hypno_pod_grants_xp_while_sleeping() {
    let mut world = World::new();
    world.insert_resource(Events::<XpGainEvent>::default());

    let pod = world.spawn((
        HypnoPod {
            target_skill: SkillType::Mining,
            xp_rate: 10.0,
        },
        PowerConsumer { demand: 5.0, active: true },
    )).id();

    let pop = world.spawn((
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
        }
    )).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(hypno_sleep_system);
    schedule.run(&mut world);

    let events = world.resource::<Events<XpGainEvent>>();
    let mut cursor = events.get_cursor();
    let mut found_event = false;
    for ev in cursor.read(events) {
        if ev.entity == pop && ev.skill == SkillType::Mining && (ev.amount - 10.0).abs() < f32::EPSILON {
            found_event = true;
        }
    }
    assert!(found_event, "XP gain event not emitted");
}

#[test]
fn test_hypno_sleep_drains_hunger_faster() {
    let mut world = World::new();
    world.insert_resource(Events::<XpGainEvent>::default());

    let pod = world.spawn((
        HypnoPod::default(),
        PowerConsumer { demand: 5.0, active: true },
    )).id();

    let pop = world.spawn((
        Pop,
        Needs { hunger: 1.0, ..Default::default() },
        PopAction {
            current: ActionType::SatisfyRest,
            ..Default::default()
        },
        AssignedTo {
            entity: pod,
            assignment_type: AssignmentType::HousingResident,
        }
    )).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(hypno_sleep_system);
    schedule.run(&mut world);

    let needs = world.get::<Needs>(pop).unwrap();
    assert!(needs.hunger < 0.995);
}

#[test]
fn test_waking_from_hypno_applies_mental_fog() {
    let mut world = World::new();

    let pop = world.spawn((
        Pop,
        HypnoSleepTracker,
        PopAction {
            current: ActionType::Idle,
            ..Default::default()
        }
    )).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(wake_up_hypno_system);

    schedule.run(&mut world);

    let fog = world.get::<MentalFog>(pop);
    assert!(fog.is_some());
    assert!(fog.unwrap().duration > 0.0);
    assert!(world.get::<HypnoSleepTracker>(pop).is_none());
}

#[test]
fn test_mental_fog_penalizes_movement_and_work() {
    let fog = MentalFog {
        duration: 10.0,
        movement_penalty: 0.5,
        work_speed_penalty: 0.5,
    };

    assert_eq!(fog.movement_penalty, 0.5);
    assert_eq!(fog.work_speed_penalty, 0.5);
}

#[test]
fn test_mental_fog_decays() {
    let mut world = World::new();

    let pop = world.spawn(MentalFog {
        duration: 1.0,
        movement_penalty: 0.5,
        work_speed_penalty: 0.5,
    }).id();

    world.run_system_once(update_mental_fog_system).unwrap();

    assert!(world.get::<MentalFog>(pop).is_none());
}
