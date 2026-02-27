#[cfg(test)]
mod tests {
    use scale::layer1::inventory::{Inventory, InventoryItem};
    use scale::layer1::items::ItemType;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_exploit_unbounded_allocation() {
        let mut inventory = Inventory::default();
        let item = InventoryItem {
            item_type: ItemType::Potato,
            entity: None,
        };

        // Attempt DoS by adding 10,000 items
        for _ in 0..10_000 {
            // Using try_add or add, both are now capped
            inventory.try_add(item.clone());
        }

        // POST-FIX: This should now fail the exploit test (len should be capped at default 20)
        assert!(inventory.items.len() <= 20, "Inventory should be capped (Exploit Prevented). Actual: {}", inventory.items.len());
        assert_eq!(inventory.items.len(), 20);
    }
}
