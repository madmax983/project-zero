use bevy::prelude::*;
use scale::layer1::architecture::embezzlement::EmbezzlementEvent;
use scale::layer1::core::chronicle::{AddChronicleEvent, Chronicle, EventImportance};
use scale::layer1::core::integration::embezzlement_chronicle_bridge;

#[test]
fn test_embezzlement_triggers_chronicle_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Chronicle>();
    app.init_resource::<Events<EmbezzlementEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, embezzlement_chronicle_bridge);

    let building = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(EmbezzlementEvent {
        target_building: building,
        embezzled_amount: 100.0,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_reader();
    let emitted: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(emitted.len(), 1, "An AddChronicleEvent should be emitted");
    assert_eq!(emitted[0].importance, EventImportance::Minor);
    assert!(emitted[0].text.contains("corrupt governor secretly embezzled 100 materials"));
}
