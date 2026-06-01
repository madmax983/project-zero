use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::solar_flare_chronicle_bridge;
use scale::layer1::nature::solar_flare_lottery::SolarFlareEvent;

#[test]
fn test_solar_flare_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<SolarFlareEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, solar_flare_chronicle_bridge);

    app.world_mut().send_event(SolarFlareEvent);

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected exactly one Chronicle event");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(
        events[0].text.contains("solar flare"),
        "Chronicle text should describe a solar flare"
    );
}
