use bevy::app::{App, Update};
use bevy::ecs::prelude::*;
use scale::layer1::actions::escape::{process_lifeboat_launches, Lifeboat};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::escape_pods_chronicle_bridge;

#[test]
fn test_escape_pods_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, (process_lifeboat_launches, escape_pods_chronicle_bridge).chain());

    let _pop = app.world_mut().spawn_empty().id();

    app.world_mut().spawn(Lifeboat {
        capacity: 1,
        occupants: vec![_pop],
        launch_triggered: true,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let events_list: Vec<_> = cursor.read(events).collect();

    assert_eq!(events_list.len(), 1);
    assert_eq!(events_list[0].text, "Escape Pods launched into orbit! The situation has become dire.");
}
