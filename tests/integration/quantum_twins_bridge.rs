use bevy::prelude::*;
use scale::layer1::biology::health::Health;
use scale::layer1::morale::Morale;
use scale::layer1::pop::PopDied;
use scale::layer1::quantum_twins::{
    handle_severance_system, propagate_damage_to_twin, update_twin_mood_system,
    update_twin_sync_system, QuantumTwin,
};
use scale::layer1::shields::DamageEvent;
use scale::layer1::skills::{Skills, XpGainEvent};
use scale::layer1::stress::StressTracker;

#[test]
fn test_quantum_twins_integration() {
    let mut app = App::new();

    app.add_event::<XpGainEvent>();
    app.add_event::<DamageEvent>();
    app.add_event::<PopDied>();

    app.add_systems(
        Update,
        (
            update_twin_sync_system,
            update_twin_mood_system,
            propagate_damage_to_twin,
            handle_severance_system,
        )
            .chain(),
    );

    let twin1 = app
        .world_mut()
        .spawn((
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            Skills::default(),
            Morale {
                value: 1.0,
                ..Default::default()
            },
            StressTracker::default(),
            QuantumTwin {
                partner: Entity::PLACEHOLDER,
                link_strength: 1.0,
            },
        ))
        .id();

    let twin2 = app
        .world_mut()
        .spawn((
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            Skills::default(),
            Morale {
                value: 1.0,
                ..Default::default()
            },
            StressTracker::default(),
            QuantumTwin {
                partner: twin1,
                link_strength: 1.0,
            },
        ))
        .id();

    app.world_mut().entity_mut(twin1).insert(QuantumTwin {
        partner: twin2,
        link_strength: 1.0,
    });

    app.update();

    assert!(app.world().get::<QuantumTwin>(twin1).is_some());
}
