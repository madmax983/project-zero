use super::setup_world;
use crate::layer1::combat::{AttackProperties, HitStop, Weapon};
use crate::layer1::execution::combat::combat_execution_system;
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::health::Health;
use crate::layer1::items::Equipment;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;

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
