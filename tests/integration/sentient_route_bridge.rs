use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer2::integration::sentient_route_chronicle_bridge;
use scale::layer2::trade::routes::{RouteComplexity, SentientTollDemandEvent};

#[test]
fn test_sentient_route_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_systems(Update, sentient_route_chronicle_bridge);

    app.init_resource::<Events<SentientTollDemandEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    let route_entity = app
        .world_mut()
        .spawn(RouteComplexity { level: 100.0 })
        .id();

    // Fire the event
    app.world_mut()
        .resource_mut::<Events<SentientTollDemandEvent>>()
        .send(SentientTollDemandEvent {
            route_id: route_entity,
            demanded_resource: "RareData".to_string(),
        });

    app.update();

    // Verify complexity reset
    let complexity = app.world().get::<RouteComplexity>(route_entity).unwrap();
    assert_eq!(complexity.level, 0.0, "Complexity should be reset to 0.0");

    // Verify chronicle event
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert!(
        events[0].text.contains("RareData"),
        "Event text should contain demanded resource"
    );
}
