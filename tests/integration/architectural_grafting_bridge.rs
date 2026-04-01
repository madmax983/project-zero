use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::grafting::{process_grafting, GraftBuildingEvent};
use scale::layer1::integration::grafting_chronicle_bridge;

#[test]
fn test_grafting_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_event::<GraftBuildingEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (process_grafting, grafting_chronicle_bridge).chain(),
    );

    let target = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(GraftBuildingEvent {
        target,
        new_module_tech_level: 2,
        efficiency_bonus: 1.5,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Should emit exactly one AddChronicleEvent for grafting"
    );
    assert_eq!(events[0].importance, EventImportance::Minor);
}
