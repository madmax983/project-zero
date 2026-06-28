use scale::layer1::core::integration::{rust_lung_chronicle_bridge_system, };
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::biology::health::Health;



// Since Rust Lung is implemented as a boolean on Health, bridging it to Chronicle effectively requires either adding a Marker component when it's true, or firing an event from the mining system. I will write a test assuming we add a marker component `RustLungContracted` when the boolean flips, to avoid spamming the chronicle.



#[test]
fn test_rust_lung_contracted_emits_chronicle() {
    let mut app = App::new();

    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, rust_lung_chronicle_bridge_system);

    let mut health = Health { current: 100.0, max: 100.0, has_rust_lung: false };

    let pop = app.world_mut().spawn(health.clone()).id();

    app.update(); // Initial spawn, has_rust_lung is false, no event

    // Simulate mining contracting rust lung
    health.has_rust_lung = true;
    app.world_mut().entity_mut(pop).insert(health);

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit exactly one AddChronicleEvent when Rust-Lung is contracted");
    assert_eq!(emitted[0].importance, EventImportance::Standard);
    assert!(emitted[0].text.contains("Rust-Lung"));
}
