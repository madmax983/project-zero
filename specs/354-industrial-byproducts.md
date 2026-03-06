# 354: Industrial Byproducts

## 1. Overview

Matter is neither created nor destroyed, only changed into something annoying.

**Industrial Byproducts** introduces the mechanic of waste generation. Crafting processes (like refining metal or cutting wood) now generate "Waste" items (e.g., Slag, Sawdust, Toxic Chemicals) in addition to the primary product. These waste items must be hauled and stored, taking up valuable stockpile capacity, or dumped. Dumping waste outside degrades the beauty and health of the surrounding tiles.

## 2. Dependencies

- `023` Refining Industry (for the base crafting loops)
- `044` Horticulture & Beauty (for Beauty value degradation)
- `025` Hauling Logistics (for moving the waste)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{Inventory, ResourceType};
    use crate::layer1::map::{TerrainGrid, BeautyGrid};
    use crate::layer1::industry::{Recipe, CraftingBuilding, complete_crafting_system};

    #[test]
    fn test_crafting_generates_primary_and_byproduct() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(complete_crafting_system);

        let building = world.spawn((
            CraftingBuilding {
                current_recipe: Some(Recipe {
                    inputs: vec![],
                    outputs: vec![(ResourceType::Metal, 1)],
                    byproducts: vec![(ResourceType::Slag, 1)],
                    work_required: 10.0,
                }),
                work_progress: 10.0, // Finished
            },
            Inventory::default(),
        )).id();

        schedule.run(&mut world);

        let inv = world.get::<Inventory>(building).unwrap();
        assert_eq!(inv.get_amount(&ResourceType::Metal), 1);
        assert_eq!(inv.get_amount(&ResourceType::Slag), 1);
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

        let mut inv = Inventory::new(5); // Max capacity of 5 total items
        inv.add(ResourceType::Slag, 5); // Full of waste

        let building = world.spawn((
            CraftingBuilding {
                current_recipe: Some(Recipe {
                    inputs: vec![],
                    outputs: vec![(ResourceType::Metal, 1)],
                    byproducts: vec![(ResourceType::Slag, 1)],
                    work_required: 10.0,
                }),
                work_progress: 10.0, // Wants to finish
            },
            inv,
        )).id();

        schedule.run(&mut world);

        // Production should NOT complete because inventory is full
        let building_state = world.get::<CraftingBuilding>(building).unwrap();
        assert_eq!(building_state.work_progress, 10.0); // Still stuck at 10.0

        let final_inv = world.get::<Inventory>(building).unwrap();
        assert_eq!(final_inv.get_amount(&ResourceType::Metal), 0); // Did not output
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::{Inventory, ResourceType};
use crate::layer1::map::BeautyGrid;

#[derive(Clone, Debug)]
pub struct Recipe {
    pub inputs: Vec<(ResourceType, u32)>,
    pub outputs: Vec<(ResourceType, u32)>,
    pub byproducts: Vec<(ResourceType, u32)>, // NEW: Byproducts
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

pub fn complete_crafting_system(
    mut query: Query<(&mut CraftingBuilding, &mut Inventory)>,
) {
    for (mut building, mut inventory) in query.iter_mut() {
        if let Some(recipe) = &building.current_recipe {
            if building.work_progress >= recipe.work_required {

                // Calculate required space: primary + byproducts
                let required_space = recipe.outputs.iter().map(|(_, amt)| amt).sum::<u32>()
                                   + recipe.byproducts.iter().map(|(_, amt)| amt).sum::<u32>();

                if inventory.has_capacity_for(required_space) {
                    for (res, amt) in &recipe.outputs {
                        inventory.add(res.clone(), *amt);
                    }
                    for (res, amt) in &recipe.byproducts {
                        inventory.add(res.clone(), *amt);
                    }

                    building.work_progress = 0.0;
                    // Note: Recipe repeats or clears depending on logic, keeping simple here
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
        let current_beauty = beauty_grid.get(action.x, action.y);
        // Decrease beauty by 1.0 per unit of waste
        beauty_grid.set(action.x, action.y, current_beauty - (action.amount as f32));

        commands.entity(entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Waste Varieties:** Differentiate waste impacts. "Sawdust" might be flammable (`FirePropagation`) but not toxic, while "Chemical Slag" might cause `Sickness` (Pop Health) to anyone walking over the dumped tile.
- **Recycling:** Integrate with future recycling tech (`221` Organic Recycling or similar) where some recipes take `Slag` as an input to produce a low-yield `Metal`.
- **Hauling AI:** Ensure Haulers treat Byproducts with equal urgency to Outputs, otherwise machines will rapidly choke on their own waste. Alternatively, allow assigning a specific "Waste Dump" stockpile that is distinct from normal storage.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code
- [ ] Crafting successfully produces both target outputs and designated byproducts.
- [ ] Machines halt production if their local inventory lacks space for the byproducts.
- [ ] Dumping waste correctly lowers the Beauty grid values on the target tile.

## 7. Technical Guidance

- Expand `ResourceType` in `src/layer1/resources.rs` to include `Slag`, `ChemicalWaste`, etc.
- In `Inventory::has_capacity_for()`, ensure it handles raw counts or volume correctly. If your project uses slots, ensure it requires $N$ slots for $N$ different item types.
- The `DumpWasteAction` should probably be an execution step in a Hauler's pathfinding routine if their target is a designated "Dumping Zone" rather than a true Stockpile.

## 8. Questions

*Builder: add questions here if spec is unclear.*
