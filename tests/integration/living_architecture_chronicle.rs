use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::living_architecture_chronicle_bridge;
use scale::layer1::architecture::living_architecture::PopConsumedEvent;

#[test]
fn test_living_architecture_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PopConsumedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, living_architecture_chronicle_bridge);

    let pop = app.world_mut().spawn_empty().id();
    let building = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(PopConsumedEvent { pop, building });
    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let ev = cursor
        .read(events)
        .next()
        .expect("Expected an AddChronicleEvent");
    assert_eq!(
        ev.text,
        "A starving living building has consumed a colonist!"
    );
}
