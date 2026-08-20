use bevy::prelude::*;
use scale::layer1::consultant::{ConsultantMarker};
use scale::layer1::ConsultantPlugin;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::consultant_chronicle_bridge;
use scale::shared::time::SimulationTime;

#[test]
fn test_consultant_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ConsultantPlugin);
    app.add_event::<AddChronicleEvent>();
    app.init_resource::<SimulationTime>();

    app.add_systems(Update, consultant_chronicle_bridge);

    app.world_mut().spawn((ConsultantMarker, Name::new("The Consultant")));

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_reader();

    let fired_events: Vec<&AddChronicleEvent> = reader.read(events).collect();

    assert_eq!(fired_events.len(), 1, "Should record Consultant arrival");
    assert_eq!(fired_events[0].importance, EventImportance::Major);
    assert!(fired_events[0].text.contains("The Consultant"));
}
