//! Security reproduction test.

use bevy_ecs::prelude::*;
use scale::layer1::combat::execute_attack;
use scale::layer1::health::Health;

#[test]
fn test_execute_attack_does_not_panic_on_despawned_entity() {
    let mut world = World::new();

    // 1. Create Attacker
    let attacker = world.spawn(scale::layer1::pop::Pop).id();

    // 2. Create Target
    let target = world.spawn(Health { current: 100.0, max: 100.0 }).id();

    // 3. Despawn Attacker
    world.despawn(attacker);

    // 4. Call execute_attack
    // This should panic because execute_attack uses world.entity_mut(attacker)
    execute_attack(&mut world, attacker, target);
}
