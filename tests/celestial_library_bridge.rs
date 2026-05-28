use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::resources::ColonyResources;
use scale::layer2::celestial_library::{CelestialLibrary, LibraryDonationEvent};
use scale::layer2::integration::celestial_library_chronicle_bridge;

#[test]
fn test_celestial_library_chronicle_bridge_sufficient() {
    let mut app = App::new();
    app.init_resource::<Events<LibraryDonationEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    // Initialize colony resources with plenty of knowledge
    let resources = ColonyResources {
        knowledge: 5000.0,
        ..Default::default()
    };
    app.insert_resource(resources);

    app.add_systems(Update, celestial_library_chronicle_bridge);

    let library = app.world_mut().spawn(CelestialLibrary {
        required_donation: 1000,
    }).id();

    // Fire the donation event
    app.world_mut().send_event(LibraryDonationEvent {
        library,
        resources_donated: 1500,
    });

    app.update();

    let final_resources = app.world().resource::<ColonyResources>();
    assert_eq!(final_resources.knowledge, 3500.0, "The knowledge must be deducted by the donated amount");

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].importance, EventImportance::Legendary);
    assert!(events[0].text.contains("1500"));
}

#[test]
fn test_celestial_library_chronicle_bridge_insufficient() {
    let mut app = App::new();
    app.init_resource::<Events<LibraryDonationEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    let resources = ColonyResources {
        knowledge: 500.0,
        ..Default::default()
    };
    app.insert_resource(resources);

    app.add_systems(Update, celestial_library_chronicle_bridge);

    let library = app.world_mut().spawn(CelestialLibrary {
        required_donation: 1000,
    }).id();

    // Fire the donation event (they try to donate 800 but only have 500)
    app.world_mut().send_event(LibraryDonationEvent {
        library,
        resources_donated: 800,
    });

    app.update();

    let final_resources = app.world().resource::<ColonyResources>();
    // Deducts what they have
    assert_eq!(final_resources.knowledge, 0.0);

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].importance, EventImportance::Standard);
    assert!(events[0].text.contains("insufficient"));
}
