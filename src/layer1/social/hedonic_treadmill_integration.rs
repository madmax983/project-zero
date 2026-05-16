use bevy_ecs::prelude::*;
use crate::layer1::social::hedonic_treadmill::ConsumeItemEvent;
use crate::layer1::social::social_mimicry::JustConsumed;
use crate::layer1::economy::items::ItemType;

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
