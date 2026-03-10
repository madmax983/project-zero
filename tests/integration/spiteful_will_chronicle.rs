use bevy::MinimalPlugins;
use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::integration::override_will_chronicle_bridge;
use scale::layer1::spiteful_will::OverrideWillEvent;

#[test]
fn test_override_will_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<OverrideWillEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, override_will_chronicle_bridge);

    let pop1 = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Events<OverrideWillEvent>>()
        .send(OverrideWillEvent {
            affected_pops: vec![pop1],
        });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = events.get_reader();
    let events_list: Vec<_> = reader.read(events).collect();

    assert_eq!(events_list.len(), 1, "Should generate one chronicle event");
    assert_eq!(events_list[0].importance, EventImportance::Major);
    assert!(
        events_list[0].text.contains("will"),
        "Text should mention the will"
    );
}
