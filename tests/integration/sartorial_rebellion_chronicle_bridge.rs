use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::sartorial_rebellion_chronicle_bridge;
use scale::layer1::social::sartorial_rebellion::{
    adopt_visual_signifier_system, enforce_dress_code_system, DressCodePolicy, FactionSignifiers,
    SignifierType,
};

#[test]
fn test_sartorial_rebellion_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<AddChronicleEvent>();
    app.init_resource::<FactionSignifiers>(); // Need to initialize resource for the system
    app.add_systems(
        Update,
        (
            enforce_dress_code_system,
            adopt_visual_signifier_system,
            sartorial_rebellion_chronicle_bridge,
        )
            .chain(),
    );

    // Initial state: no policy
    app.update();

    // Add policy banning RedBandana
    app.insert_resource(DressCodePolicy {
        banned_signifiers: vec![SignifierType::RedBandana],
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = reader.read(events).collect();

    assert_eq!(
        emitted.len(),
        1,
        "Should emit one AddChronicleEvent for banning RedBandana"
    );
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(
        emitted[0].text.contains("banned the wearing of RedBandana"),
        "Event text should describe the ban"
    );

    // Update policy to also ban BackwardsCap
    app.world_mut()
        .resource_mut::<DressCodePolicy>()
        .banned_signifiers
        .push(SignifierType::BackwardsCap);

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = reader.read(events).collect();

    assert_eq!(emitted.len(), 2, "Should emit ONE more AddChronicleEvent (total 2) since the resource changed but we track previously banned items");
    assert!(
        emitted[1].text.contains("BackwardsCap"),
        "Event text should mention the new banned item"
    );
}
