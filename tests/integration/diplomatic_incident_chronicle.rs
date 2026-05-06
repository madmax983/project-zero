use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, Chronicle, EventImportance};
use scale::layer1::core::integration::diplomatic_incident_chronicle_bridge;
use scale::layer1::law::embassy::DiplomaticIncidentEvent;
use scale::layer1::FactionId;

#[test]
fn diplomatic_incident_triggers_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<DiplomaticIncidentEvent>();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(Chronicle::default());

    app.add_systems(Update, diplomatic_incident_chronicle_bridge);

    app.world_mut()
        .resource_mut::<Events<DiplomaticIncidentEvent>>()
        .send(DiplomaticIncidentEvent {
            faction_id: FactionId::MinersGuild,
            reason: "Arrested ambassador".to_string(),
        });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("Diplomatic Incident"));
}
