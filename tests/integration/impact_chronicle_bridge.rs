use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::impact_strike_chronicle_bridge;
use scale::layer1::environment::impact::ImpactStrikeEvent;
use scale::layer1::map::GridPosition;

#[test]
fn test_impact_strike_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<AddChronicleEvent>();
    app.add_event::<ImpactStrikeEvent>();

    app.add_systems(Update, impact_strike_chronicle_bridge);

    app.world_mut().send_event(ImpactStrikeEvent {
        center: GridPosition { x: 15, y: 30 },
        radius: 5,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Should emit exactly one AddChronicleEvent for the impact strike"
    );

    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("15"));
    assert!(events[0].text.contains("30"));
}
