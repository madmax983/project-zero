use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer2::cryo_mutiny::MutineerPop;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::integration::cryo_mutiny_chronicle_bridge;

#[test]
fn test_cryo_mutiny_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, cryo_mutiny_chronicle_bridge);

    app.world_mut().spawn(MutineerPop { culture_shock: 100.0 });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(matches!(emitted[0].importance, EventImportance::Major));
    assert!(emitted[0].text.contains("Cryo-Mutiny"));
}
