use bevy_ecs::prelude::*;
use scale::layer1::geology::tectonic::{
    check_quake_system, update_stress_system, MegaQuakeEvent, TectonicStress,
};
use scale::layer1::geology::GeologicalEvent;
use scale::layer1::map::GridPosition;
use scale::layer1::resources::MiningEvent;
use scale::layer1::environment::volatile::ExplosionEvent;

#[test]
fn test_mining_and_explosions_cause_mega_quake() {
    let mut app = bevy_app::App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.init_resource::<TectonicStress>();
    app.add_event::<MiningEvent>();
    app.add_event::<ExplosionEvent>();
    app.add_event::<MegaQuakeEvent>();
    app.add_event::<GeologicalEvent>();

    app.add_systems(
        bevy_app::Update,
        (
            update_stress_system,
            check_quake_system.after(update_stress_system),
        ),
    );

    app.update(); // Initialize

    // Send a bunch of MiningEvents
    for _ in 0..150 {
        app.world_mut().send_event(MiningEvent { amount: 10.0 });
    }

    app.update();

    let stress = app.world().resource::<TectonicStress>();
    // Expect stress to be added (150 * 0.5 = 75)
    // - 0.1 dissipation = 74.9
    assert_eq!(stress.current, 74.9);

    // Send an explosion event
    app.world_mut().send_event(ExplosionEvent {
        center: GridPosition { x: 5, y: 5 },
        damage: 300.0, // 300 * 0.1 = 30
        radius: 2,
    });

    app.update();

    // Now stress was 74.9 + 30 = 104.9.
    // This is >= 100.0, so MegaQuake should trigger and stress should be reset to 0.0.
    let stress_after = app.world().resource::<TectonicStress>();
    assert_eq!(stress_after.current, 0.0);

    let mut mega_quake_events = app.world_mut().resource_mut::<Events<MegaQuakeEvent>>();
    assert_eq!(mega_quake_events.len(), 1);
    mega_quake_events.clear();

    let geo_events = app.world().resource::<Events<GeologicalEvent>>();
    assert_eq!(geo_events.len(), 1);
}
