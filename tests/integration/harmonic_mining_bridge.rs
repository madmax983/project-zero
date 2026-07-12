use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::harmonic_mining_chronicle_bridge;
use scale::layer1::core::map::GridPosition;
use scale::layer1::execution::mining::harmonic_mining::{Frequency, TriggerSonicDrillEvent};

#[test]
fn test_harmonic_mining_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<TriggerSonicDrillEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, harmonic_mining_chronicle_bridge);

    app.world_mut()
        .resource_mut::<Events<TriggerSonicDrillEvent>>()
        .send(TriggerSonicDrillEvent {
            center: GridPosition { x: 0, y: 0 },
            radius: 3,
            frequency: Frequency::Iron,
        });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("Harmonic Mining"));
}
