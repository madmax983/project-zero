use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::accidental_terraforming::{BiomeShiftEvent, BiomeType};
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::accidental_terraforming_chronicle_bridge;

#[test]
fn test_accidental_terraforming_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<BiomeShiftEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, accidental_terraforming_chronicle_bridge);

    let planet = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(BiomeShiftEvent {
        planet,
        old_biome: BiomeType::Ice,
        new_biome: BiomeType::Ocean,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut count = 0;
    for event in reader.read(events) {
        assert_eq!(event.importance, EventImportance::Major);
        count += 1;
    }
    assert_eq!(count, 1);
}
