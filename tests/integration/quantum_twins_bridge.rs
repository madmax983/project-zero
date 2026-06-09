use bevy::prelude::*;
use scale::layer1::biology::health::Health;
use scale::layer1::pop::Pop;
use scale::layer1::quantum_twins::QuantumTwin;
use scale::layer1::shields::{DamageEvent, KineticBarrier};
use scale::layer1::systems::Layer1SystemSet;

#[test]
fn test_damage_event_propagates_through_shields_to_twin() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Register events and system set
    app.add_event::<DamageEvent>();
    app.configure_sets(Update, Layer1SystemSet::Consumption);

    // Just register the specific systems from our seam
    app.add_systems(
        Update,
        (
            scale::layer1::shields::apply_damage_with_shields,
            scale::layer1::quantum_twins::propagate_damage_to_twin
                .after(scale::layer1::shields::apply_damage_with_shields),
        )
            .in_set(Layer1SystemSet::Consumption),
    );

    // Twin A gets hit. Has a shield!
    let twin_a = app
        .world_mut()
        .spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            KineticBarrier {
                power: 10.0,
                max_power: 50.0,
                velocity_threshold: 10.0,
            },
        ))
        .id();

    // Twin B doesn't get hit directly, and doesn't have a shield.
    let twin_b = app
        .world_mut()
        .spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            QuantumTwin {
                partner: twin_a,
                link_strength: 1.0,
            },
        ))
        .id();

    // Link Twin A to Twin B
    app.world_mut().entity_mut(twin_a).insert(QuantumTwin {
        partner: twin_b,
        link_strength: 1.0,
    });

    // Send a massive damage event that overflows the shield
    app.world_mut().send_event(DamageEvent {
        target: twin_a,
        amount: 25.0,
        velocity: 50.0, // fast, hits shield
    });

    app.update();

    let health_a = app.world().get::<Health>(twin_a).unwrap();
    let health_b = app.world().get::<Health>(twin_b).unwrap();
    let barrier = app.world().get::<KineticBarrier>(twin_a).unwrap();

    // The shield took 10 damage and died.
    assert_eq!(barrier.power, 0.0, "Barrier should be depleted");

    // The pop took the remaining 15 damage. Health is 85.
    assert_eq!(health_a.current, 85.0, "Twin A should take remaining 15 damage");

    // The current spec of `propagate_damage_to_twin` reads the raw DamageEvent amount.
    // It propagates the *original* 25 damage, NOT the mitigated 15 damage.
    // Twin B will take 25 damage. Health is 75.
    assert_eq!(health_b.current, 75.0, "Twin B should receive the original DamageEvent amount (25)");
}
