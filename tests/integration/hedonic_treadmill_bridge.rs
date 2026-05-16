use bevy::prelude::*;
use scale::layer1::social::hedonic_treadmill::ConsumeItemEvent;
use scale::layer1::economy::items::ItemType;
use scale::layer1::pop::Pop;
use scale::layer1::social::social_mimicry::JustConsumed;
use scale::layer1::social::hedonic_treadmill_integration::hedonic_treadmill_consumption_bridge;

#[test]
fn test_hedonic_treadmill_consumption_bridge() {
    let mut app = App::new();
    app.add_event::<ConsumeItemEvent>();

    app.add_systems(Update, hedonic_treadmill_consumption_bridge);

    let pop_id = app.world_mut().spawn((
        Pop,
        JustConsumed { item: ItemType::LuxuryMeal },
    )).id();

    app.update();

    let events = app.world().resource::<Events<ConsumeItemEvent>>();
    let mut reader = events.get_cursor();
    let emitted_events: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted_events.len(), 1);
    assert_eq!(emitted_events[0].consumer, pop_id);
    assert_eq!(emitted_events[0].item_quality, 5.0); // LuxuryMeal quality
}
