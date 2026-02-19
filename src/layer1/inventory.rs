use crate::layer1::items::ItemType;
use bevy_ecs::prelude::*;

/// An item in an inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct InventoryItem {
    pub item_type: ItemType,
}

impl Default for InventoryItem {
    fn default() -> Self {
        Self {
            item_type: ItemType::default(),
        }
    }
}

/// Component for storing personal items (tools, curios, etc.).
#[derive(Component, Debug, Default, Clone)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
}

impl Inventory {
    /// Adds an item to the inventory.
    pub fn add(&mut self, item: InventoryItem) {
        self.items.push(item);
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
        };
        inventory.add(item.clone());
        assert_eq!(inventory.items.len(), 1);
        assert_eq!(inventory.items[0], item);
    }
}
