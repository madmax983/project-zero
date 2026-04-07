import re

with open("src/layer1/local_tributes.rs", "r") as f:
    data = f.read()

# Replace Dummy Definitions with imports
imports = """use crate::layer1::economy::inventory::Inventory;
use crate::layer1::economy::items::ItemType;
use crate::layer1::disasters::DisasterEvent;"""

data = data.replace("use bevy::prelude::*;", f"use bevy::prelude::*;\n{imports}")

# Remove the dummy structures
data = re.sub(r'// Dummy ItemType.*?\n}\n', '', data, flags=re.DOTALL)
# One more try for the dummy struct
dummy_block = """// Dummy ItemType and Inventory for compilation of example
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemType { Food, Energy }
#[derive(Resource)]
pub struct Inventory { items: std::collections::HashMap<ItemType, u32> }
impl Default for Inventory {
    fn default() -> Self { Self::new() }
}

impl Inventory {
    pub fn new() -> Self { Self { items: std::collections::HashMap::new() } }
    pub fn add(&mut self, item: ItemType, amount: u32) { *self.items.entry(item).or_insert(0) += amount; }
    pub fn get(&self, item: ItemType) -> u32 { *self.items.get(&item).unwrap_or(&0) }
    pub fn remove(&mut self, item: ItemType, amount: u32) -> bool {
        let current = self.get(item);
        if current >= amount {
            self.items.insert(item, current - amount);
            true
        } else {
            false
        }
    }
}"""
data = data.replace(dummy_block, "")

# Actually we need to rewrite how we interact with Inventory in leviathan_appeasement_system and in the tests
# because the real Inventory has a different API:
# pub struct InventoryItem { pub item_type: ItemType, pub entity: Option<Entity> }
# pub struct Inventory { pub items: Vec<InventoryItem>, pub capacity: usize }
# It doesn't have an easy remove method by item type and amount in the same way.
# But since this is a global tribute logic we need to iterate over the items vec and remove matching items.
with open("src/layer1/local_tributes.rs", "w") as f:
    f.write(data)
