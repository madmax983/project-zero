use bevy::prelude::*;
use scale::layer1::biology::health::Health;
use scale::layer1::quantum_twins::{propagate_damage_to_twin, QuantumTwin};
use scale::layer1::shields::{apply_damage_with_shields, DamageEvent, KineticBarrier};

#[test]
fn test_damage_propagation_and_shielding_seam() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add required event
    app.add_event::<DamageEvent>();

    // Register the systems in the specific chained order to ensure twin damage gets duplicated
    // *before* the shield system absorbs the primary target's damage.
    app.add_systems(Update, (
        propagate_damage_to_twin,
        apply_damage_with_shields,
    ).chain());

    // Target A: The primary target, has a shield
    let target_a = app.world_mut().spawn((
        Health {
            current: 100.0,
            max: 100.0,
            has_rust_lung: false,
        },
        KineticBarrier {
            power: 20.0,
            max_power: 50.0,
            velocity_threshold: 10.0,
        },
    )).id();

    // Target B: The twin, no shield
    let target_b = app.world_mut().spawn((
        Health {
            current: 100.0,
            max: 100.0,
            has_rust_lung: false,
        },
        QuantumTwin {
            partner: target_a,
            link_strength: 1.0,
        },
    )).id();

    // Link Target A to Target B
    app.world_mut().entity_mut(target_a).insert(QuantumTwin {
        partner: target_b,
        link_strength: 1.0,
    });

    // Fire a fast projectile at Target A for 50 damage
    app.world_mut().send_event(DamageEvent {
        target: target_a,
        amount: 50.0,
        velocity: 50.0, // Triggers shield
    });

    app.update();

    let health_a = app.world().get::<Health>(target_a).unwrap();
    let shield_a = app.world().get::<KineticBarrier>(target_a).unwrap();
    let health_b = app.world().get::<Health>(target_b).unwrap();

    // Target A's shield absorbs 20, letting 30 through to health
    assert_eq!(shield_a.power, 0.0, "Target A's shield should be depleted");
    assert_eq!(health_a.current, 70.0, "Target A should take 30 damage (50 - 20 shield)");

    // Target B receives the raw damage event (50) and has no shield
    assert_eq!(health_b.current, 50.0, "Target B should take the full 50 damage propagated from A");
}
