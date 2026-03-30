#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::spiteful_will::InheritanceEvent;
    use scale::layer1::integration::inheritance_chronicle_bridge;
    use scale::layer1::inventory::InventoryItem;
    use scale::layer1::items::ItemType;

    #[test]
    fn inheritance_event_emits_standard_chronicle() {
        let mut app = bevy_app::App::new();
        app.add_event::<InheritanceEvent>();
        app.add_event::<AddChronicleEvent>();

        app.add_systems(bevy_app::Update, inheritance_chronicle_bridge);

        let deceased = app.world_mut().spawn_empty().id();
        let item_entity = app.world_mut().spawn_empty().id();

        app.world_mut().send_event(InheritanceEvent {
            deceased,
            items: vec![InventoryItem { item_type: ItemType::Tool, entity: Some(item_entity) }],
        });

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_cursor();
        let events: Vec<_> = reader.read(chronicle_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].importance, EventImportance::Standard);
        assert!(events[0].text.contains("will"));
    }
}
