use bevy::prelude::*;
use scale::layer1::artists_muse::ArtWork;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::artists_muse_chronicle_bridge;
use scale::layer1::crafting::Quality;

#[test]
fn test_artists_muse_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, artists_muse_chronicle_bridge);

    app.update(); // clear startup

    // Spawn an ArtWork with Quality::Masterpiece
    app.world_mut().spawn(ArtWork {
        item_type: "Sculpture".to_string(),
        quality: Quality::Masterpiece,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(
        emitted.len(),
        1,
        "Should emit exactly one AddChronicleEvent when a Masterpiece ArtWork is produced"
    );
    assert!(
        emitted[0]
            .text
            .contains("A tortured artist has created a masterpiece born from suffering."),
        "Chronicle event text should mention the masterpiece"
    );
}
