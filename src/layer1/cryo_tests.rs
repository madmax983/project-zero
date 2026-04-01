use crate::layer1::building::{Building, BuildingType};
use crate::layer1::cryo::{
    cryo_sickness_decay_system, enter_cryo_system, exit_cryo_system, CryoSickness, CryoStasis,
};
use crate::layer1::map::GridPosition;
use crate::layer1::needs::{decay_needs_system, Needs};
use crate::layer1::pop::{Pop, Speed};
use bevy_ecs::system::RunSystemOnce;

#[test]
fn test_cryo_stasis_halts_need_decay() {
    let mut world = crate::setup::setup_world();
    // Spawn normal pop
    let pop1 = world
        .spawn((
            Pop,
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 0.8,
                oxygen: 100.0,
            },
        ))
        .id();
    // Spawn frozen pop
    let pop2 = world
        .spawn((
            Pop,
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 0.8,
                oxygen: 100.0,
            },
            CryoStasis,
        ))
        .id();

    // Run decay system multiple times
    for _ in 0..100 {
        let _ = world.run_system_once(decay_needs_system);
    }

    let needs1 = world.get::<Needs>(pop1).unwrap();
    let needs2 = world.get::<Needs>(pop2).unwrap();

    assert!(needs1.hunger < 1.0, "Normal pop should decay");
    assert_eq!(needs2.hunger, 1.0, "Frozen pop should NOT decay");
}

#[test]
fn test_enter_cryo_applies_component() {
    let mut world = crate::setup::setup_world();
    let pop = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();
    let _pod = world
        .spawn((
            Building {
                building_type: BuildingType::CryoPod,
            },
            GridPosition { x: 5, y: 5 },
            // Needs a component to trigger entry, e.g., CryoOrder
            crate::layer1::cryo::CryoOrder { target: pop },
        ))
        .id();

    let _ = world.run_system_once(enter_cryo_system);

    assert!(world.get::<CryoStasis>(pop).is_some());

    // Check pop moved to pod
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 5);
    assert_eq!(pos.y, 5);
}

#[test]
fn test_enter_cryo_cancels_action() {
    use crate::layer1::utility_types::{ActionType, PopAction};
    let mut world = crate::setup::setup_world();
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            PopAction {
                current: ActionType::Work,
                current_utility: 1.0,
                ticks_committed: 10,
            },
        ))
        .id();

    let _pod = world
        .spawn((
            Building {
                building_type: BuildingType::CryoPod,
            },
            GridPosition { x: 5, y: 5 },
            crate::layer1::cryo::CryoOrder { target: pop },
        ))
        .id();

    let _ = world.run_system_once(enter_cryo_system);

    let action = world.get::<PopAction>(pop).unwrap();
    assert_eq!(action.current, ActionType::Idle);
    assert!(action.current_utility < f32::EPSILON);
}

#[test]
fn test_exit_cryo_applies_sickness() {
    let mut world = crate::setup::setup_world();
    let pop = world
        .spawn((
            Pop,
            CryoStasis,
            Speed {
                base: 1.0,
                current: 1.0,
                accumulator: 0.0,
            },
        ))
        .id();

    // Trigger exit (e.g. remove CryoStasis or use a command)
    // For this test, we assume a system handles the transition if marked
    world.entity_mut(pop).insert(crate::layer1::cryo::ThawOrder);

    let _ = world.run_system_once(exit_cryo_system);

    assert!(world.get::<CryoStasis>(pop).is_none());
    assert!(world.get::<CryoSickness>(pop).is_some());

    // Check speed penalty
    let speed = world.get::<Speed>(pop).unwrap();
    assert!(speed.current < 1.0);
}

#[test]
fn test_cryo_sickness_decays() {
    let mut world = crate::setup::setup_world();
    let pop = world
        .spawn((
            Pop,
            CryoSickness {
                duration: 10,
                severity: 0.5,
            },
            Speed {
                base: 1.0,
                current: 0.5,
                accumulator: 0.0,
            },
        ))
        .id();

    let _ = world.run_system_once(cryo_sickness_decay_system);

    let sickness = world.get::<CryoSickness>(pop).unwrap();
    assert_eq!(sickness.duration, 9);

    // Run until expiration
    for _ in 0..10 {
        let _ = world.run_system_once(cryo_sickness_decay_system);
    }

    assert!(world.get::<CryoSickness>(pop).is_none());
    let speed = world.get::<Speed>(pop).unwrap();
    assert_eq!(speed.current, 1.0, "Speed should recover");
}
