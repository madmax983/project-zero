# 394 - Hauling Logistics

## 1. Overview
The "Hauling Logistics" system forces players to manage the physical movement of resources. Items produced on the ground (like mined ore or harvested logs) cannot be used directly by builders or crafters until they are moved to a designated `Stockpile`. This creates bottlenecks and requires assigning Pops specifically to hauling tasks, making layout efficiency and worker allocation critical.

## 2. Dependencies
- `016-utility-ai-system.md` (Utility AI for hauling task selection)
- `022-resource-stockpiles.md` (Stockpile zones)
- `009-job-system.md` (Job assignments)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::inventory::Inventory;
    use crate::layer1::item::{Item, ItemType};
    use crate::layer1::map::TerrainGrid;

    #[test]
    fn test_builder_cannot_use_ground_item() {
        let mut app = App::new();
        app.add_systems(Update, check_building_materials);

        let pop = app.world_mut().spawn((
            Builder,
            Job { task: "Build Wall", ..default() },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        // Spawn a loose item on the ground, not in a stockpile
        app.world_mut().spawn((
            Item(ItemType::Wood),
            LooseItem, // Not in a stockpile
            Transform::from_xyz(1.0, 0.0, 0.0),
        ));

        app.update();

        // The builder should not be able to find valid materials
        let builder = app.world().get::<Builder>(pop).unwrap();
        assert!(!builder.has_materials, "Builder cannot use loose items");
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
    }

    #[test]
    fn test_utility_ai_scores_hauling() {
        let mut app = App::new();
        app.add_systems(Update, score_hauling_action);

        let pop = app.world_mut().spawn((
            Hauler,
            UtilityScores::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        // Spawn a loose item
        let item = app.world_mut().spawn((
            Item(ItemType::Wood),
            LooseItem,
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Spawn a valid stockpile with space
        let stockpile = app.world_mut().spawn((
            Stockpile { capacity: 10, current: 0 },
            Transform::from_xyz(-1.0, 0.0, 0.0),
        )).id();

        app.update();

        // Utility AI should strongly prefer hauling this item
        let scores = app.world().get::<UtilityScores>(pop).unwrap();
        let haul_score = scores.get("HaulItem");
        assert!(haul_score > 0.5, "Hauler should score hauling high when items are loose and stockpiles have space");
    }

    #[test]
    fn test_hauling_completes_when_reached() {
        let mut app = App::new();
        app.add_systems(Update, execute_hauling_task);

        let pop = app.world_mut().spawn((
            Hauler,
            CarryingItem(ItemType::Wood),
            Transform::from_xyz(-1.0, 0.0, 0.0), // At stockpile
        )).id();

        let stockpile = app.world_mut().spawn((
            Stockpile { capacity: 10, current: 0 },
            Inventory::default(),
            Transform::from_xyz(-1.0, 0.0, 0.0),
        )).id();

        app.update();

        // Item should be transferred to stockpile inventory
        let inv = app.world().get::<Inventory>(stockpile).unwrap();
        assert_eq!(inv.count(&ItemType::Wood), 1, "Item should be deposited into stockpile");

        let pop_carrying = app.world().get::<CarryingItem>(pop);
        assert!(pop_carrying.is_none(), "Pop should no longer be carrying the item");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct LooseItem;

#[derive(Component)]
pub struct Stockpile {
    pub capacity: i32,
    pub current: i32,
}

#[derive(Component)]
pub struct Builder {
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
    pub has_materials: bool,
}

#[derive(Component)]
pub struct Hauler;

#[derive(Component)]
pub struct CarryingItem(pub ItemType);

pub fn check_building_materials(
    mut query: Query<&mut Builder>,
    stockpiles: Query<(&Stockpile, &Inventory)>,
) {
    // Only check stockpiles, completely ignore loose items
    let mut total_wood = 0;
    for (_, inv) in stockpiles.iter() {
        total_wood += inv.count(&ItemType::Wood);
    }

    for mut builder in query.iter_mut() {
        if total_wood >= 10 { // e.g., Wall requires 10 Wood
            builder.has_materials = true;
        } else {
            builder.has_materials = false;
        }
    }
}

pub fn score_hauling_action(
    mut query: Query<(&Hauler, &Transform, &mut UtilityScores)>,
    loose_items: Query<&Transform, With<LooseItem>>,
    stockpiles: Query<&Stockpile>,
) {
    let has_space = stockpiles.iter().any(|s| s.current < s.capacity);
    let has_items = !loose_items.is_empty();

    if has_space && has_items {
        for (_, _, mut scores) in query.iter_mut() {
            scores.set("HaulItem", 0.8);
        }
    } else {
        for (_, _, mut scores) in query.iter_mut() {
            scores.set("HaulItem", 0.0);
        }
    }
}

pub fn execute_hauling_task(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Transform, &CarryingItem), With<Hauler>>,
    mut stockpiles: Query<(&Transform, &mut Stockpile, &mut Inventory), Without<Hauler>>,
) {
    for (pop_entity, pop_transform, carrying) in pops.iter_mut() {
        for (stockpile_transform, mut stockpile, mut inv) in stockpiles.iter_mut() {
            if pop_transform.translation.distance(stockpile_transform.translation) < 0.1 {
                if stockpile.current < stockpile.capacity {
                    // Deposit
                    inv.add(carrying.0, 1);
                    stockpile.current += 1;
                    commands.entity(pop_entity).remove::<CarryingItem>();
                    break;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding & AI Action**: The hauling task should be a full state machine (`FindItem -> MoveToItem -> PickupItem -> FindStockpile -> MoveToStockpile -> DropItem`). The green implementation simplifies this for the deposit step.
- **Stockpile Filters**: Stockpiles need filtering so haulers know exactly *which* stockpile accepts Wood vs. Stone.
- **Carrying Capacity**: Pops should only carry a limited number of items, perhaps scaled by Strength or a Backpack tool.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥ 85% for `layer1/logistics.rs` or `layer1/hauling.rs`
- [ ] Builders cannot use `LooseItem` entities directly from the ground.
- [ ] Utility AI correctly scores hauling tasks when loose items exist and stockpiles have space.

## 7. Technical Guidance
- `CarryingItem` should be added dynamically when a Pop picks up a `LooseItem`, which then despawns the loose entity.
- The `check_building_materials` function should ideally interact with the existing crafting/building system.

## 8. Questions
*Builder: add questions here if spec is unclear.*
