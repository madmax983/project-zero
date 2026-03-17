use crate::layer1::items::ItemType;
use bevy_ecs::prelude::*;

/// An item in an inventory.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct InventoryItem {
    /// The specific type of the item (e.g., Potato, Curio).
    pub item_type: ItemType,
    /// The optional entity associated with this item (if preserved).
    pub entity: Option<Entity>,
}

/// A global resource for tracking unassigned, stored items.
#[derive(Resource, Default, Debug, Clone)]
pub struct ColonyInventory {
    /// Maps ItemType to quantity.
    pub items: std::collections::HashMap<ItemType, u32>,
}

impl ColonyInventory {
    /// Adds items to the colony inventory.
    pub fn add_item(&mut self, item_type: ItemType, count: u32) {
        *self.items.entry(item_type).or_insert(0) += count;
    }

    /// Gets the count of a specific item type.
    #[must_use]
    pub fn get_count(&self, item_type: &ItemType) -> u32 {
        *self.items.get(item_type).unwrap_or(&0)
    }
}

/// Component for storing personal items (tools, curios, etc.).
#[derive(Component, Debug, Clone)]
pub struct Inventory {
    /// The list of items currently held in the inventory.
    pub items: Vec<InventoryItem>,
    /// The maximum number of items this inventory can hold.
    pub capacity: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            capacity: 20,
        }
    }
}

impl Inventory {
    /// Adds an item to the inventory.
    ///
    /// # Deprecated
    /// Use [`Inventory::try_add`] to handle capacity limits safely.
    /// This method will panic or silently fail in future versions if full.
    /// Currently, it just calls `try_add` and ignores the result (simulating old behavior but safe).
    pub fn add(&mut self, item: InventoryItem) {
        let _ = self.try_add(item);
    }

    /// Tries to add an item to the inventory.
    ///
    /// Returns `true` if added, `false` if inventory is full.
    pub fn try_add(&mut self, item: InventoryItem) -> bool {
        if self.items.len() >= self.capacity {
            return false;
        }
        self.items.push(item);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::items::ItemType;

    #[test]
    fn test_inventory_add() {
        let mut inventory = Inventory::default();
        let item = InventoryItem {
            item_type: ItemType::Potato,
            entity: None,
        };
        inventory.add(item.clone());
        assert_eq!(inventory.items.len(), 1);
        assert_eq!(inventory.items[0], item);
    }

    #[test]
    fn test_inventory_capacity() {
        let mut inventory = Inventory {
            capacity: 2,
            ..Default::default()
        };
        let item = InventoryItem {
            item_type: ItemType::Potato,
            entity: None,
        };

        assert!(inventory.try_add(item.clone()));
        assert!(inventory.try_add(item.clone()));
        assert!(!inventory.try_add(item.clone())); // Should fail
        assert_eq!(inventory.items.len(), 2);
    }

    #[test]
    fn test_colony_inventory_add_and_get() {
        let mut inventory = ColonyInventory::default();
        inventory.add_item(ItemType::FormalWear, 5);
        inventory.add_item(ItemType::FormalWear, 3);
        inventory.add_item(ItemType::Potato, 10);

        assert_eq!(inventory.get_count(&ItemType::FormalWear), 8);
        assert_eq!(inventory.get_count(&ItemType::Potato), 10);
        assert_eq!(inventory.get_count(&ItemType::Tool), 0);
    }
}
