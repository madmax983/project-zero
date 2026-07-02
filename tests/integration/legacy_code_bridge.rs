use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::reformat_chronicle_bridge;
use scale::layer1::tech::legacy_code::ReformatCommand;

#[test]
fn test_reformat_chronicle_bridge() {
    let mut app = bevy_app::App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<ReformatCommand>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, reformat_chronicle_bridge);

    let core_entity = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Events<ReformatCommand>>()
        .send(ReformatCommand { core_entity });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(chronicle_events.len(), 1, "Expected one Chronicle event");
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();
    assert!(events[0].text.contains("total reformat"));
}
