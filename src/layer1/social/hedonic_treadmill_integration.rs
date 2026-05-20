use crate::layer1::economy::items::ItemType;
use crate::layer1::social::hedonic_treadmill::ConsumeItemEvent;
use crate::layer1::social::social_mimicry::JustConsumed;
use bevy_ecs::prelude::*;

pub fn get_item_quality(item: &ItemType) -> f32 {
    match item {
        ItemType::Potato | ItemType::MysteryMeal => 1.0,
        ItemType::Wheat | ItemType::Rice | ItemType::Corn | ItemType::Fruit => 1.5,
        ItemType::Soy => 1.2,
        ItemType::Meat | ItemType::Fish | ItemType::AlienMeatA => 2.0,
        ItemType::AlienMeatB => 2.5,
        ItemType::GlowMushroom => 3.0,
        ItemType::LuxuryMeal => 5.0,
        ItemType::VoidAle => 4.0,
        ItemType::Rations => 0.5,
        _ => 1.0,
    }
}

pub fn hedonic_treadmill_consumption_bridge(
    query: Query<(Entity, &JustConsumed), Added<JustConsumed>>,
    mut events: EventWriter<ConsumeItemEvent>,
) {
    for (entity, consumed) in query.iter() {
        events.send(ConsumeItemEvent {
            consumer: entity,
            item_quality: get_item_quality(&consumed.item),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_item_quality_table() {
        let cases = vec![
            (ItemType::Potato, 1.0),
            (ItemType::MysteryMeal, 1.0),
            (ItemType::Wheat, 1.5),
            (ItemType::Meat, 2.0),
            (ItemType::AlienMeatB, 2.5),
            (ItemType::GlowMushroom, 3.0),
            (ItemType::VoidAle, 4.0),
            (ItemType::LuxuryMeal, 5.0),
            (ItemType::Rations, 0.5),
            (ItemType::Tool, 1.0), // fallback case
            (ItemType::None, 1.0), // fallback case
        ];

        for (item_type, expected_quality) in cases {
            assert_eq!(
                get_item_quality(&item_type),
                expected_quality,
                "Failed on item {:?}",
                item_type
            );
        }
    }

    #[test]
    fn test_hedonic_treadmill_consumption_bridge() {
        let mut world = World::new();
        world.init_resource::<Events<ConsumeItemEvent>>();

        let consumer_id = world
            .spawn(JustConsumed {
                item: ItemType::LuxuryMeal,
            })
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hedonic_treadmill_consumption_bridge);
        schedule.run(&mut world);

        let events = world.resource::<Events<ConsumeItemEvent>>();
        let mut reader = events.get_cursor();
        let mut event_count = 0;

        for event in reader.read(events) {
            event_count += 1;
            assert_eq!(event.consumer, consumer_id);
            assert_eq!(event.item_quality, 5.0); // LuxuryMeal quality
        }

        assert_eq!(event_count, 1);
    }
}
