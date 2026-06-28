//! Crafting byproducts and waste management.
//!
//! When items are crafted, they often produce secondary materials (byproducts)
//! like scrap or waste. This module defines how these byproducts are handled
//! and stored in an `Inventory`. If an inventory reaches capacity, crafting is blocked.
//!
//! ## Important Note on Capacity
//! `Inventory` capacity applies to the **sum of all items**. If an inventory is full
//! of waste, new primary outputs cannot be created until the waste is dumped.

use crate::layer1::beauty::BeautyGrid;
use crate::layer1::economy::resources::ResourceType;
use bevy_ecs::prelude::*;

/// A component representing a container for resources.
///
/// Inventories have a strict `capacity`. Production systems must check
/// [`Inventory::has_capacity_for`] before adding new items or byproducts.
///
/// ## Examples
/// ```
/// use scale::layer1::crafting::byproducts::Inventory;
/// use scale::layer1::economy::resources::ResourceType;
///
/// let mut inv = Inventory { items: vec![], capacity: 10 };
/// inv.add(ResourceType::Metal, 5);
/// assert_eq!(inv.get_amount(&ResourceType::Metal), 5);
/// assert!(inv.has_capacity_for(5));
/// assert!(!inv.has_capacity_for(6));
/// ```
#[derive(Component, Default, Clone, Debug)]
pub struct Inventory {
    /// The stored items and their quantities.
    pub items: Vec<(ResourceType, u32)>,
    /// The maximum total amount of all items combined that this inventory can hold.
    pub capacity: u32,
}

impl Inventory {
    /// Retrieves the current stored amount of a specific `ResourceType`.
    pub fn get_amount(&self, res: &ResourceType) -> u32 {
        self.items
            .iter()
            .find(|(r, _)| r == res)
            .map(|(_, amt)| *amt)
            .unwrap_or(0)
    }

    /// Adds a quantity of a `ResourceType` to the inventory.
    ///
    /// ## Panics
    /// This function does *not* panic if capacity is exceeded. It is the caller's
    /// responsibility to verify capacity using `has_capacity_for` prior to adding.
    pub fn add(&mut self, res: ResourceType, amount: u32) {
        if let Some(existing) = self.items.iter_mut().find(|(r, _)| r == &res) {
            existing.1 += amount;
        } else {
            self.items.push((res, amount));
        }
    }

    /// Checks if the inventory can accommodate the specified `amount` of new items.
    ///
    /// Evaluates the sum of all currently stored items against the `capacity`.
    pub fn has_capacity_for(&self, amount: u32) -> bool {
        let current: u32 = self.items.iter().map(|(_, amt)| amt).sum();
        current + amount <= self.capacity
    }
}

#[derive(Clone, Debug)]
pub struct Recipe {
    pub inputs: Vec<(ResourceType, u32)>,
    pub outputs: Vec<(ResourceType, u32)>,
    pub byproducts: Vec<(ResourceType, u32)>,
    pub work_required: f32,
}

#[derive(Component)]
pub struct CraftingBuilding {
    pub current_recipe: Option<Recipe>,
    pub work_progress: f32,
}

#[derive(Component)]
pub struct DumpWasteAction {
    pub x: i32,
    pub y: i32,
    pub amount: u32,
}

pub fn complete_crafting_system(mut query: Query<(&mut CraftingBuilding, &mut Inventory)>) {
    for (mut building, mut inventory) in query.iter_mut() {
        if let Some(recipe) = &building.current_recipe {
            if building.work_progress >= recipe.work_required {
                let required_space = recipe.outputs.iter().map(|(_, amt)| amt).sum::<u32>()
                    + recipe.byproducts.iter().map(|(_, amt)| amt).sum::<u32>();

                if inventory.has_capacity_for(required_space) {
                    for (res, amt) in &recipe.outputs {
                        inventory.add(*res, *amt);
                    }
                    for (res, amt) in &recipe.byproducts {
                        inventory.add(*res, *amt);
                    }
                    building.work_progress = 0.0;
                }
            }
        }
    }
}

pub fn waste_dumping_system(
    mut commands: Commands,
    mut beauty_grid: ResMut<BeautyGrid>,
    actions: Query<(Entity, &DumpWasteAction)>,
) {
    for (entity, action) in actions.iter() {
        if let (Ok(ux), Ok(uy)) = (usize::try_from(action.x), usize::try_from(action.y)) {
            let current_beauty = beauty_grid.get(ux, uy);
            #[allow(clippy::cast_precision_loss)]
            beauty_grid.set(ux, uy, current_beauty - (action.amount as f32));
        }
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::beauty::BeautyGrid;
    use crate::layer1::economy::resources::ResourceType;

    #[test]
    fn test_crafting_generates_primary_and_byproduct() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(complete_crafting_system);

        let building = world
            .spawn((
                CraftingBuilding {
                    current_recipe: Some(Recipe {
                        inputs: vec![],
                        outputs: vec![(ResourceType::Metal, 1)],
                        byproducts: vec![(ResourceType::Waste, 1)],
                        work_required: 10.0,
                    }),
                    work_progress: 10.0, // Finished
                },
                Inventory {
                    capacity: 100,
                    items: vec![],
                },
            ))
            .id();

        schedule.run(&mut world);

        let inv = world.get::<Inventory>(building).unwrap();
        assert_eq!(inv.get_amount(&ResourceType::Metal), 1);
        assert_eq!(inv.get_amount(&ResourceType::Waste), 1);
    }

    #[test]
    fn test_dumping_waste_reduces_beauty() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(waste_dumping_system);

        let mut beauty_grid = BeautyGrid::new(10, 10);
        beauty_grid.set(5, 5, 10.0); // Initial beauty
        world.insert_resource(beauty_grid);

        // Action to dump waste at (5, 5)
        world.spawn(DumpWasteAction {
            x: 5,
            y: 5,
            amount: 5,
        });

        schedule.run(&mut world);

        let grid = world.resource::<BeautyGrid>();
        assert!(grid.get(5, 5) < 10.0); // Beauty is reduced by the dumped waste
    }

    #[test]
    fn test_stockpile_capacity_blocks_production_if_full_of_waste() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(complete_crafting_system);

        let mut inv = Inventory {
            capacity: 5,
            items: vec![],
        }; // Max capacity of 5 total items
        inv.add(ResourceType::Waste, 5); // Full of waste

        let building = world
            .spawn((
                CraftingBuilding {
                    current_recipe: Some(Recipe {
                        inputs: vec![],
                        outputs: vec![(ResourceType::Metal, 1)],
                        byproducts: vec![(ResourceType::Waste, 1)],
                        work_required: 10.0,
                    }),
                    work_progress: 10.0, // Wants to finish
                },
                inv,
            ))
            .id();

        schedule.run(&mut world);

        // Production should NOT complete because inventory is full
        let building_state = world.get::<CraftingBuilding>(building).unwrap();
        assert_eq!(building_state.work_progress, 10.0); // Still stuck at 10.0

        let final_inv = world.get::<Inventory>(building).unwrap();
        assert_eq!(final_inv.get_amount(&ResourceType::Metal), 0); // Did not output
    }
}
