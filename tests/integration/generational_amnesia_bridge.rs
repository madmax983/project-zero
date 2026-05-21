use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::generational_amnesia_chronicle_bridge;
use scale::layer1::entities::pop::Pop;
use scale::layer1::psychology::generational_amnesia::GenerationalAmnesia;
use scale::shared::time::SimulationTime;

#[test]
fn test_amnesia_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<AddChronicleEvent>();

    app.insert_resource(SimulationTime {
        tick: 0,
        speed: scale::shared::time::SimSpeed::Normal,
    });

    app.add_systems(Update, generational_amnesia_chronicle_bridge);

    app.world_mut().spawn((
        Pop,
        GenerationalAmnesia {
            decay_rate: 0.1,
            current_amnesia: 95.0, // High amnesia
        },
    ));

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).cloned().collect();

    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(emitted[0].text.contains("amnesia") || emitted[0].text.contains("forgotten"));
}
