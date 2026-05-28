use bevy::prelude::*;
use scale::layer1::combat::{execute_attack, CombatState, Weapon, AttackProperties};
use scale::layer1::health::Health;
use scale::layer1::items::Equipment;
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::architecture::fossilized_fleet::FossilizedShip;

#[test]
fn test_fossilized_ship_defense_bonus_reduces_damage() {
    let mut world = World::new();
    world.insert_resource(scale::shared::time::SimulationTime::default());

    let sword = world.spawn(Weapon {
        properties: AttackProperties {
            damage: 100.0,
            range: 1.0,
            cooldown: 5,
            accuracy: 1.0,
        }
    }).id();

    let attacker = world.spawn((
        CombatState::default(),
        Equipment { weapon: Some(sword), ..Default::default() },
        Pop,
    )).id();

    let target = world.spawn((
        Health { current: 100.0, max: 100.0, has_rust_lung: false },
        GridPosition { x: 10, y: 10 },
    )).id();

    world.spawn((
        FossilizedShip {
            decay_rate: 1.0,
            maintenance_cost: 10,
            defense_bonus: 40,
            structural_integrity: 100.0,
        },
        GridPosition { x: 10, y: 10 },
    ));

    execute_attack(&mut world, attacker, target);

    let health = world.get::<Health>(target).unwrap();

    assert_eq!(health.current, 40.0);
}
