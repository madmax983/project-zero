#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::core::chronicle::AddChronicleEvent;
    use scale::layer1::core::integration::tether_stump_lost_tech_bridge;
    use scale::layer1::economy::inventory::{Inventory, InventoryItem};
    use scale::layer1::economy::items::ItemType;
    use scale::layer1::resources::ColonyResources;

    #[test]
    fn test_lost_tech_recovered_from_inventory() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.add_event::<AddChronicleEvent>();

        let mut resources = ColonyResources::default();
        resources.max_knowledge = 1000.0;
        resources.knowledge = 0.0;
        app.insert_resource(resources);

        app.add_systems(Update, tether_stump_lost_tech_bridge);

        let mut inventory = Inventory::default();
        inventory.try_add(InventoryItem {
            item_type: ItemType::LostTech,
            entity: None,
        });

        app.world_mut().spawn(inventory);

        app.update();

        // Verify Knowledge granted
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.knowledge, 50.0);

        // Verify Chronicle Event emitted
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1);

        // Verify Lost Tech removed from inventory
        let mut query = app.world_mut().query::<&Inventory>();
        let inv = query.single(app.world());
        assert_eq!(inv.items.len(), 0);
    }
}
