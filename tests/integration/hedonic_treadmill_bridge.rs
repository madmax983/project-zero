use bevy::app::{App, Update};
use bevy::ecs::prelude::*;
use bevy::ecs::event::Events;
use scale::layer1::agriculture::farm::consume_food_system;
use scale::layer1::economy::items::ItemType;
use scale::layer1::economy::resources::ColonyResources;
use scale::layer1::entities::pop::Pop;
use scale::layer1::needs::Needs;
use scale::layer1::social::hedonic_treadmill::ConsumeItemEvent;

#[test]
fn test_consume_food_emits_consume_item_event() {
    let mut app = App::new();

    app.add_event::<ConsumeItemEvent>();
    app.insert_resource(ColonyResources {
        food: 10.0,
        ..Default::default()
    });

    app.add_systems(Update, consume_food_system);

    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            Needs {
                hunger: 0.0, // hungry
                ..Default::default()
            },
        ))
        .id();

    app.update();

    let events = app.world().resource::<Events<ConsumeItemEvent>>();
    let mut cursor = events.get_cursor();
    let emitted_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted_events.len(), 1, "Should emit exactly one ConsumeItemEvent");
    assert_eq!(emitted_events[0].consumer, pop_entity);
}
