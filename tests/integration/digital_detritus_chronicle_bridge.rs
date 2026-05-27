use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer3::digital_detritus::{record_virus_event_chronicle_system, VirusEvent};

#[test]
fn test_digital_detritus_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<VirusEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, record_virus_event_chronicle_system);

    app.world_mut()
        .resource_mut::<Events<VirusEvent>>()
        .send(VirusEvent);

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0].text.contains("Critical Virus"));
}
