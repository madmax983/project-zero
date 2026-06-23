use bevy::prelude::*;
use scale::layer1::cassandra_syndrome::{CultLeader, DoomsdayWarningEvent};
use scale::layer1::environment::disasters::DisasterType;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::cassandra_syndrome_chronicle_bridge;

#[test]
fn test_doomsday_warning_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<DoomsdayWarningEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, cassandra_syndrome_chronicle_bridge);

    // Act
    app.world_mut()
        .resource_mut::<Events<DoomsdayWarningEvent>>()
        .send(DoomsdayWarningEvent {
            prophet_entity: Entity::from_raw(1),
            disaster_type: DisasterType::MassiveEarthquake,
        });

    app.update();

    // Assert
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one Chronicle event");
    assert_eq!(
        events[0].importance,
        EventImportance::Standard,
        "Warning event should be Standard importance"
    );
    assert!(
        events[0]
            .text
            .contains("A Doomsday Warning has been issued"),
        "Event text should match"
    );
}

#[test]
fn test_cult_formation_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<DoomsdayWarningEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, cassandra_syndrome_chronicle_bridge);

    // Let the first update clear trackers if needed
    app.update();
    app.world_mut().clear_trackers();

    // Act: Spawn a pop with CultLeader
    app.world_mut().spawn(CultLeader);

    app.update();

    // Assert
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one Chronicle event");
    assert_eq!(
        events[0].importance,
        EventImportance::Major,
        "Cult formation event should be Major importance"
    );
    assert!(
        events[0]
            .text
            .contains("A cult has formed around a prophetic leader"),
        "Event text should match"
    );
}
