use bevy_app::App;
use bevy_ecs::event::Events;
use bevy::prelude::Name;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::digital_immortality::{Ghost, MindUploadEvent};
use scale::layer1::core::integration::digital_immortality_chronicle_bridge;

#[test]
fn test_mind_upload_emits_chronicle() {
    let mut app = App::new();

    app.add_event::<MindUploadEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(bevy_app::Update, digital_immortality_chronicle_bridge);

    let pop_entity = app
        .world_mut()
        .spawn((Name::new("Evelyn"), Ghost))
        .id();

    let mainframe_entity = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Events<MindUploadEvent>>()
        .send(MindUploadEvent {
            target_pop: pop_entity,
            destination_mainframe: mainframe_entity,
        });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("Evelyn"));
    assert!(events[0].text.contains("digital immortality"));
}
