use bevy_ecs::prelude::*;
use bevy::prelude::*;
use scale::layer1::architecture::sunk_cost_monument::{
    calculate_sunk_cost_upkeep_system, handle_monument_cancellation_system,
    SunkCostUpkeep, MonumentMarker, CancelConstructionEvent
};
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::{sunk_cost_chronicle_bridge, sunk_cost_resource_drain_system};
use scale::layer1::economy::resources::ColonyResources;

#[test]
fn test_sunk_cost_resource_drain() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut resources = ColonyResources::default();
    resources.stone = 1000.0;
    app.insert_resource(resources);

    app.add_systems(Update, (
        calculate_sunk_cost_upkeep_system,
        sunk_cost_resource_drain_system,
    ).chain());

    app.world_mut().spawn(SunkCostUpkeep {
        base_cost: 10.0,
        multiplier: 1.1,
        ticks_building: 0,
    });

    app.update(); // Tick 1: ticks_building -> 1, cost = 10 * 1.1^1 = 11.0. 1000 - 11 = 989

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.stone, 989.0);
}

#[test]
fn test_sunk_cost_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<CancelConstructionEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, (
        handle_monument_cancellation_system,
        sunk_cost_chronicle_bridge,
    ).chain());

    let monument_id = app.world_mut().spawn((
        MonumentMarker,
        Transform::from_translation(Vec3::ZERO),
    )).id();

    app.world_mut().send_event(CancelConstructionEvent(monument_id));

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();

    let events: Vec<_> = reader.read(chronicle_events).collect();
    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(events[0].importance, EventImportance::Legendary);
    assert!(events[0].text.contains("Sunk-Cost Monument was abandoned"));
}
