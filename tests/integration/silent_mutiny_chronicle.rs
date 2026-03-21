use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::integration::sensor_glitch_chronicle_bridge_system;
use scale::layer2::silent_mutiny::SensorGlitchEvent;

#[test]
fn test_sensor_glitch_chronicle_bridge_system() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<SensorGlitchEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, sensor_glitch_chronicle_bridge_system);

    let dummy_fleet = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(SensorGlitchEvent {
        fleet: dummy_fleet,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_reader();
    let emitted: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(
        emitted.len(),
        1,
        "Should emit exactly one AddChronicleEvent for the sensor glitch"
    );

    assert_eq!(
        emitted[0].importance,
        EventImportance::Standard,
        "Sensor glitch event should have Standard importance"
    );
    assert!(
        emitted[0].text.contains("sensor glitch") || emitted[0].text.contains("anomalous"),
        "Event text should describe the sensor glitch. Got: {}",
        emitted[0].text
    );
}
