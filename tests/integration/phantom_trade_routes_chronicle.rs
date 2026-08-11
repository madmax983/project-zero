use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::integration::phantom_trade_routes_chronicle_bridge;
use scale::layer2::trade::phantom_trade_routes::FixPhantomRouteEvent;

#[test]
fn test_phantom_trade_routes_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<Events<FixPhantomRouteEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, phantom_trade_routes_chronicle_bridge);

    app.world_mut().send_event(FixPhantomRouteEvent {
        entity: Entity::PLACEHOLDER,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected one chronicle event");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert_eq!(
        events[0].text,
        "The realization of Pointless Labor shattered their morale."
    );
}
