use bevy::math::Vec3;
use bevy::prelude::Transform;
use bevy_app::App;
use bevy_ecs::event::Events;
use bevy_ecs::prelude::*;
use scale::layer1::architecture::sunk_cost_monument::{
    calculate_sunk_cost_upkeep_system, handle_monument_cancellation_system,
    CancelConstructionEvent, MonumentMarker, SunkCostUpkeep,
};
use scale::layer1::core::integration::sunk_cost_resource_drain_system;
use scale::layer1::economy::resources::{ColonyResources, ResourceType};
use scale::shared::time::SimulationTime;

#[test]
fn test_sunk_cost_drains_resources_successfully() {
    let mut app = App::new();

    app.add_event::<CancelConstructionEvent>();
    app.insert_resource(SimulationTime::default());

    let resources = ColonyResources {
        metal: 100.0,
        ..ColonyResources::zeroed()
    };
    app.insert_resource(resources);

    app.add_systems(
        bevy_app::Update,
        (
            calculate_sunk_cost_upkeep_system,
            sunk_cost_resource_drain_system,
        )
            .chain(),
    );

    let _monument = app
        .world_mut()
        .spawn((
            MonumentMarker,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            SunkCostUpkeep {
                base_cost: 10.0,
                multiplier: 1.0,
                ticks_building: 0,
            },
        ))
        .id();

    // Act
    app.update();

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.get_amount(ResourceType::Metal), 90.0);

    let cancel_events = app.world().resource::<Events<CancelConstructionEvent>>();
    let mut cursor = cancel_events.get_cursor();
    assert_eq!(cursor.read(cancel_events).count(), 0);
}

#[test]
fn test_sunk_cost_cancels_when_insufficient_resources() {
    let mut app = App::new();

    app.add_event::<CancelConstructionEvent>();
    app.insert_resource(SimulationTime::default());

    // Insufficient resources (only 5 metal, need 10)
    let resources = ColonyResources {
        metal: 5.0,
        ..ColonyResources::zeroed()
    };
    app.insert_resource(resources);

    app.add_systems(
        bevy_app::Update,
        (
            calculate_sunk_cost_upkeep_system,
            sunk_cost_resource_drain_system,
            handle_monument_cancellation_system,
        )
            .chain(),
    );

    let _monument = app
        .world_mut()
        .spawn((
            MonumentMarker,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            SunkCostUpkeep {
                base_cost: 10.0,
                multiplier: 1.0,
                ticks_building: 0,
            },
        ))
        .id();

    // Act
    app.update();

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(
        resources.get_amount(ResourceType::Metal),
        5.0,
        "Should not deduct partial resources"
    );

    // Monument should be despawned (canceled)
    assert!(
        app.world().get_entity(_monument).is_err(),
        "Monument should be despawned when canceled"
    );
}
