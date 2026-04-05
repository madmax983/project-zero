use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, Chronicle};
use scale::layer1::integration::temporal_stutter_chronicle_bridge;
use scale::layer1::temporal_ghost_towns::TemporalStutterEvent;

#[test]
fn test_temporal_stutter_triggers_chronicle_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Chronicle>();
    app.init_resource::<Events<TemporalStutterEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, temporal_stutter_chronicle_bridge);

    let building = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(TemporalStutterEvent {
        entity: building,
        duration: 10,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    assert!(
        !chronicle_events.is_empty(),
        "Temporal stutter should trigger a chronicle event"
    );
}
