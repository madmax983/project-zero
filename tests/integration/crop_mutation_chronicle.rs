use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::genetics::{CropMutationEvent, MutationType};
use scale::layer1::integration::crop_mutation_chronicle_bridge;

#[test]
fn test_aggressive_growth_mutation_emits_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CropMutationEvent>();
    app.add_event::<AddChronicleEvent>();

    let crop_entity = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(CropMutationEvent {
        crop_entity,
        mutation_type: MutationType::AggressiveGrowth,
    });

    app.add_systems(Update, crop_mutation_chronicle_bridge);
    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert!(
        events[0].text.contains("aggressive growth")
            || events[0].text.contains("Aggressive Growth"),
        "Text should mention aggressive growth"
    );
    assert_eq!(events[0].importance, EventImportance::Major);
}

#[test]
fn test_toxic_spores_mutation_emits_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CropMutationEvent>();
    app.add_event::<AddChronicleEvent>();

    let crop_entity = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(CropMutationEvent {
        crop_entity,
        mutation_type: MutationType::ToxicSpores,
    });

    app.add_systems(Update, crop_mutation_chronicle_bridge);
    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert!(
        events[0].text.contains("toxic spores") || events[0].text.contains("Toxic Spores"),
        "Text should mention toxic spores"
    );
    assert_eq!(events[0].importance, EventImportance::Major);
}
