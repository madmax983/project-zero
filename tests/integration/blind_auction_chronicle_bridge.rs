use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::auction::{VaultOpenedEvent, VaultOutcome};
use scale::layer2::integration::blind_auction_chronicle_bridge_system;

#[test]
fn test_blind_auction_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<VaultOpenedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, blind_auction_chronicle_bridge_system);

    // Act: TechBoost outcome
    app.world_mut().send_event(VaultOpenedEvent {
        outcome: VaultOutcome::TechBoost,
    });
    app.update();

    {
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut cursor = events.get_cursor();
        let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

        assert_eq!(
            emitted.len(),
            1,
            "Should emit exactly one AddChronicleEvent for TechBoost"
        );
        assert_eq!(emitted[0].importance, EventImportance::Major);
        assert!(emitted[0].text.contains("TechBoost"));
    }

    app.world_mut().resource_mut::<Events<AddChronicleEvent>>().clear();

    // Act: CatastrophicAnomaly outcome
    app.world_mut().send_event(VaultOpenedEvent {
        outcome: VaultOutcome::CatastrophicAnomaly,
    });
    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted2: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(
        emitted2.len(),
        1,
        "Should emit exactly one AddChronicleEvent for CatastrophicAnomaly"
    );
    assert_eq!(emitted2[0].importance, EventImportance::Legendary);
    assert!(emitted2[0].text.contains("CatastrophicAnomaly"));
}
