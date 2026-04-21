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

    /// Checks if the inventory contains an item of the specified type.
    pub fn has_item(&self, item_type: ItemType) -> bool {
        self.items.iter().any(|item| item.item_type == item_type)
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
}
