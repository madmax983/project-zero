use super::setup_world;
use crate::layer1::combat::{AttackProperties, HitStop, Weapon};
use crate::layer1::execution::combat::combat_execution_system;
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::health::Health;
use crate::layer1::items::Equipment;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::utility_types::{ActionType, PopAction};

#[test]
fn test_combat_execution_system_attacks_in_range() {
    let mut world = setup_world();

    let enemy = world
        .spawn((
            GridPosition { x: 1, y: 0 },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    let weapon = world
        .spawn(Weapon {
            properties: AttackProperties {
                damage: 10.0,
                range: 1.0,
                cooldown: 0,
                accuracy: 1.0,
            },
        })
        .id();

    world.spawn((
        Pop,
        GridPosition { x: 0, y: 0 },
        Equipment {
            weapon: Some(weapon),
            ..Default::default()
        },
        MovementTarget {
            target_entity: enemy,
            target_position: GridPosition { x: 1, y: 0 },
            for_action: ActionType::Fight,
        },
    ));

    combat_execution_system(&mut world);

    let health = world.get::<Health>(enemy).unwrap();
    let dmg = 100.0 - health.current;
    assert!(
        (dmg - 10.0).abs() < f32::EPSILON || (dmg - 20.0).abs() < f32::EPSILON,
        "Damage should be 10.0 or 20.0 (crit), got {}",
        dmg
    );
}

#[test]
fn test_combat_execution_system_chases_out_of_range() {
    let mut world = setup_world();

    let enemy = world
        .spawn((
            GridPosition { x: 5, y: 0 },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    let weapon = world
        .spawn(Weapon {
            properties: AttackProperties {
                damage: 10.0,
                range: 1.0,
                cooldown: 0,
                accuracy: 1.0,
            },
        })
        .id();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Equipment {
                weapon: Some(weapon),
                ..Default::default()
            },
            MovementTarget {
                target_entity: enemy,
                target_position: GridPosition { x: 5, y: 0 },
                for_action: ActionType::Fight,
            },
            AtTarget,
        ))
        .id();

    combat_execution_system(&mut world);

    let health = world.get::<Health>(enemy).unwrap();
    assert!((health.current - 100.0).abs() < f32::EPSILON);

    assert!(world.get::<AtTarget>(pop).is_none());
}

#[test]
fn test_combat_execution_blocked_by_hit_stop() {
    let mut world = setup_world();

    let enemy = world
        .spawn((
            GridPosition { x: 1, y: 0 },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    let weapon = world
        .spawn(Weapon {
            properties: AttackProperties {
                damage: 10.0,
                range: 1.0,
                cooldown: 0,
                accuracy: 1.0,
            },
        })
        .id();

    world.spawn((
        Pop,
        GridPosition { x: 0, y: 0 },
        Equipment {
            weapon: Some(weapon),
            ..Default::default()
        },
        MovementTarget {
            target_entity: enemy,
            target_position: GridPosition { x: 1, y: 0 },
            for_action: ActionType::Fight,
        },
        HitStop { ticks_remaining: 1 },
    ));

    combat_execution_system(&mut world);

    let health = world.get::<Health>(enemy).unwrap();
    assert!(
        (health.current - 100.0).abs() < f32::EPSILON,
        "HitStop should prevent attack"
    );
}

#[test]
fn test_combat_cleanup_on_target_despawn() {
    let mut world = setup_world();

    // Spawn Enemy
    let enemy = world
        .spawn((GridPosition { x: 1, y: 0 }, Health::default()))
        .id();

    // Spawn Pop targeting Enemy
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Equipment::default(),
            MovementTarget {
                target_entity: enemy,
                target_position: GridPosition { x: 1, y: 0 },
                for_action: ActionType::Fight,
            },
            AtTarget, // Simulate being in range/combat
            PopAction {
                current: ActionType::Fight,
                current_utility: 100.0,
                ticks_committed: 10,
            },
        ))
        .id();

    // Despawn Enemy
    world.despawn(enemy);

    // Run System
    combat_execution_system(&mut world);

    // Verify Cleanup
    assert!(
        world.get::<MovementTarget>(pop).is_none(),
        "MovementTarget should be removed"
    );
    assert!(
        world.get::<AtTarget>(pop).is_none(),
        "AtTarget should be removed"
    );

    let action = world.get::<PopAction>(pop).unwrap();
    assert_eq!(
        action.current,
        ActionType::Idle,
        "Action should reset to Idle"
    );
    assert!((action.current_utility - 0.0).abs() < f32::EPSILON);
    assert_eq!(action.ticks_committed, 1);
}

#[test]
fn test_combat_updates_target_position() {
    let mut world = setup_world();

    // Spawn Enemy at (1, 0)
    let enemy = world
        .spawn((GridPosition { x: 1, y: 0 }, Health::default()))
        .id();

    // Spawn Pop targeting Enemy at (1, 0)
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Equipment::default(),
            MovementTarget {
                target_entity: enemy,
                target_position: GridPosition { x: 1, y: 0 },
                for_action: ActionType::Fight,
            },
        ))
        .id();

    // Move Enemy to (2, 0)
    if let Some(mut pos) = world.get_mut::<GridPosition>(enemy) {
        pos.x = 2;
    }

    // Run System
    combat_execution_system(&mut world);

    // Verify MovementTarget update
    let mt = world.get::<MovementTarget>(pop).unwrap();
    assert_eq!(
        mt.target_position,
        GridPosition { x: 2, y: 0 },
        "Target position should update"
    );
}

#[test]
fn test_combat_unarmed_defaults() {
    let mut world = setup_world();

    // Enemy at range 1
    let enemy = world
        .spawn((
            GridPosition { x: 1, y: 0 },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    // Pop Unarmed
    // Default range should be 1.0, so (0,0) to (1,0) is in range.
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Equipment::default(), // No weapon
            MovementTarget {
                target_entity: enemy,
                target_position: GridPosition { x: 1, y: 0 },
                for_action: ActionType::Fight,
            },
        ))
        .id();

    // Run System
    combat_execution_system(&mut world);

    // Verify:
    // 1. Should be AtTarget (because default range 1.0 covers dist 1.0)
    assert!(
        world.get::<AtTarget>(pop).is_some(),
        "Unarmed pop should be in range (default 1.0)"
    );

    // 2. Damage should be 0 (current implementation for unarmed)
    let health = world.get::<Health>(enemy).unwrap();
    assert!(
        (health.current - 100.0).abs() < f32::EPSILON,
        "Unarmed attack currently does 0 damage"
    );
}

#[test]
fn test_combat_removes_at_target_when_out_of_range() {
    let mut world = setup_world();

    // Enemy moved far away (5, 0)
    let enemy = world
        .spawn((GridPosition { x: 5, y: 0 }, Health::default()))
        .id();

    // Pop at (0, 0) with AtTarget (simulate previously in range)
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Equipment::default(),
            MovementTarget {
                target_entity: enemy,
                target_position: GridPosition { x: 5, y: 0 },
                for_action: ActionType::Fight,
            },
            AtTarget,
        ))
        .id();

    // Run System
    combat_execution_system(&mut world);

    // Verify AtTarget removed
    assert!(
        world.get::<AtTarget>(pop).is_none(),
        "AtTarget should be removed when out of range"
    );
}
