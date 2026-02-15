# 120: Crop Diversity

## Overview

Currently, farms produce generic "Food". This spec introduces specific **Crop Types** (Wheat, Potato, Rice) that farmers can plant. Each crop has unique properties:
- **Wheat**: High yield, standard growth, sensitive to winter.
- **Potato**: Low yield, fast growth, cold resistant.
- **Rice**: Medium yield, requires water (future), fast growth.

This adds a strategic layer to farming:
1.  **Selection**: Players choose crops based on season/needs.
2.  **Palette Fatigue**: Different crops count as different meal types, mitigating the "Boring Diet" debuff (Spec 114).
3.  **Risk Management**: Potatoes are a safety net against winter starvation.

## Dependencies

- `008` — Farm Building (provides `Farm` component and `produce_food_system`)
- `027` — Seasonal Rhythms (provides `SeasonState`)
- `114` — Palette Fatigue (provides `ItemType` enum to expand)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/farm_crop_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::farm::{Farm, produce_food_system};
    use crate::layer1::items::ItemType;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_farm_has_selected_crop() {
        let farm = Farm::default();
        // Default crop should be Wheat (standard)
        assert_eq!(farm.selected_crop, ItemType::Wheat);
    }

    #[test]
    fn test_colony_resources_has_specific_crops() {
        let mut res = ColonyResources::default();
        // These fields should exist
        res.wheat = 10.0;
        res.potato = 5.0;
        res.rice = 2.0;

        assert_eq!(res.wheat, 10.0);
        assert_eq!(res.potato, 5.0);
        assert_eq!(res.rice, 2.0);

        // "food" getter should return sum
        // Note: If 'food' field remains as legacy/sum, test that.
        // Assuming we migrate 'food' to be a getter or sum of fields.
        // For MVP, we might keep 'food' as a cache or just sum it.
        // Let's assume `total_food()` method.
        // assert_eq!(res.total_food(), 17.0);
    }

    #[test]
    fn test_produce_food_wheat_yield() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SeasonState { current_season: Season::Spring }); // Good weather

        // Spawn Farm with Wheat
        world.spawn((
            Farm { selected_crop: ItemType::Wheat, ..Default::default() },
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction { current: ActionType::Farm, ..Default::default() },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        // Wheat base yield is high (e.g. 0.006 vs standard 0.005)
        assert!(res.wheat > 0.005);
        assert_eq!(res.potato, 0.0);
    }

    #[test]
    fn test_produce_food_potato_winter_resistance() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SeasonState { current_season: Season::Winter });

        // Spawn Farm with Potato
        world.spawn((
            Farm { selected_crop: ItemType::Potato, ..Default::default() },
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction { current: ActionType::Farm, ..Default::default() },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();

        // Potato in winter (0.8 modifier) vs Wheat in winter (0.2 modifier)
        // Potato base (0.004) * 0.8 = 0.0032
        // Wheat base (0.006) * 0.2 = 0.0012

        assert!(res.potato > 0.003);
    }

    #[test]
    fn test_change_crop_selection() {
        let mut farm = Farm::default();
        assert_eq!(farm.selected_crop, ItemType::Wheat);

        farm.selected_crop = ItemType::Rice;
        assert_eq!(farm.selected_crop, ItemType::Rice);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ItemType` (`src/layer1/items.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ItemType {
    #[default]
    None,
    // Existing
    Tool,
    // New Crops
    Wheat,
    Potato,
    Rice,
    // Legacy mapping? Or generic 'Food' is removed/deprecated?
    // We keep 'Food' in ColonyResources as abstraction if needed,
    // but specific items are better for palette fatigue.
}
```

### 2. Update `ColonyResources` (`src/layer1/resources.rs`)

```rust
pub struct ColonyResources {
    // ... existing ...
    // Add specific crops
    pub wheat: f32,
    pub potato: f32,
    pub rice: f32,

    // Legacy 'food' might become a computed property or summary
    // For backward compatibility, maybe keep 'food' and sync it?
    // Better: migrate consumers to use `total_food()`.
    // But for MVP: update `add_food` to just add to specific fields AND update a cached 'food' total?
    // Let's stick to adding fields and updating consumers.

    // Helper
    pub fn total_food(&self) -> f32 {
        self.wheat + self.potato + self.rice + self.rations + /* legacy food if any */ 0.0
    }
}
```

### 3. Update `Farm` Component (`src/layer1/farm.rs`)

```rust
#[derive(Component)]
pub struct Farm {
    pub capacity: usize,
    pub workers: Vec<Entity>,
    // New field
    pub selected_crop: ItemType,
}

impl Default for Farm {
    fn default() -> Self {
        Self {
            capacity: 2,
            workers: Vec::new(),
            selected_crop: ItemType::Wheat, // Default crop
        }
    }
}
```

### 4. Update `produce_food_system` (`src/layer1/farm.rs`)

```rust
// Define stats
struct CropStats {
    base_yield: f32,
    winter_modifier: f32, // 1.0 = immune, 0.0 = dies
}

fn get_crop_stats(crop: ItemType) -> CropStats {
    match crop {
        ItemType::Wheat => CropStats { base_yield: 0.006, winter_modifier: 0.2 },
        ItemType::Potato => CropStats { base_yield: 0.004, winter_modifier: 0.8 },
        ItemType::Rice => CropStats { base_yield: 0.005, winter_modifier: 0.5 },
        _ => CropStats { base_yield: 0.005, winter_modifier: 0.5 }, // Fallback
    }
}

pub fn produce_food_system(
    farm_query: Query<(&Building, &GridPosition, &Farm)>, // Add Farm to read selection
    // ... params
) {
    // ... setup ...

    for (building, pos, farm) in farm_query.iter() {
        // ... worker loop ...

        let stats = get_crop_stats(farm.selected_crop);
        let season_mod = match season {
             Season::Winter => stats.winter_modifier,
             _ => 1.0, // Or other seasonal logic
        };

        let production = efficiency * stats.base_yield * season_mod;

        if production > 0.0 {
            let res = &mut resources;
            match farm.selected_crop {
                ItemType::Wheat => res.wheat += production,
                ItemType::Potato => res.potato += production,
                ItemType::Rice => res.rice += production,
                _ => res.add_food(production), // Fallback to generic
            }
            // CRITICAL: Update generic `food` field for backward compatibility until all consumers use `total_food()`
            // If `add_food` is used above, it already increments `food`.
            // If specific fields are used, we MUST manually increment `food` to prevent starvation.
            if farm.selected_crop != ItemType::None {
                res.food += production;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **UI Integration**: Add a cycle button in the Farm inspection window to switch `selected_crop`.
- **Consumption**: Update `consume_food_system` to eat specific crops.
    - Logic: Eat perishable first? Or eat most abundant?
    - For Palette Fatigue: Eat what wasn't eaten recently.
- **Seeds**: Future spec could require `Seeds` resource to switch crops (Spec 118 Heirloom Seeds?).
- **Legacy Migration**: If `ColonyResources.food` is used everywhere, add a getter/setter wrapper or refactor all usages to `total_food()`.

## Acceptance Criteria

- [ ] `Farm` defaults to `Wheat`.
- [ ] `ColonyResources` tracks `wheat`, `potato`, `rice`.
- [ ] Production logic respects `selected_crop` stats.
- [ ] Potatoes perform better in Winter than Wheat.
- [ ] Wheat yields more in Spring/Summer than Potatoes.
- [ ] Tests pass.

## Technical Guidance

- In `consume_food_system`, try to consume from the largest pile first to balance stocks, OR random weighted.
- Remember to update `ItemType` in `src/layer1/items.rs` first as it's a dependency.
