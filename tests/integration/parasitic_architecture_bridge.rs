use bevy::prelude::*;
use scale::layer1::parasitic_architecture::BuildingConsumedEvent;
use scale::layer1::integration::parasitic_architecture_chronicle_bridge;
use scale::layer1::chronicle::{Chronicle, AddChronicleEvent};

#[test]
fn test_megastructure_consumption_triggers_chronicle_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Chronicle>();
    app.init_resource::<Events<BuildingConsumedEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, parasitic_architecture_chronicle_bridge);

    let building = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(BuildingConsumedEvent {
        entity: building,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    assert!(!chronicle_events.is_empty(), "Building consumption should trigger a chronicle event");
}
