# 022: Resource Stockpiles

## Overview

Implement resource storage limits (`max_food`, `max_wood`, `max_stone`) and a `Stockpile` building that increases these limits. This adds a logistical constraint to the game: players cannot hoard infinite resources without building infrastructure.

## Dependencies

- `018` — Mining and Resources (for `ColonyResources`)
- `006` — Building Placement (for `Building` and `BuildingType`)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/stockpile_tests.rs (or inside resources.rs tests module)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::stockpile::{Stockpile, update_resource_caps_system};

    #[test]
    fn test_colony_resources_caps_default() {
        let resources = ColonyResources::default();
        // Default small capacity for survival
        assert_eq!(resources.max_food, 50.0);
        assert_eq!(resources.max_wood, 50.0);
        assert_eq!(resources.max_stone, 20.0);
    }

    #[test]
    fn test_add_resource_clamped_to_max() {
        let mut resources = ColonyResources {
            wood: 40.0,
            max_wood: 50.0,
            ..Default::default()
        };

        // Add 20, should cap at 50
        resources.add_wood(20.0);
        assert_eq!(resources.wood, 50.0);
    }

    #[test]
    fn test_add_resource_no_overflow() {
        let mut resources = ColonyResources {
            stone: 20.0,
            max_stone: 20.0,
            ..Default::default()
        };

        resources.add_stone(10.0);
        assert_eq!(resources.stone, 20.0);
    }

    #[test]
    fn test_stockpile_component_defaults() {
        let stockpile = Stockpile::default();
        assert_eq!(stockpile.food_bonus, 0.0);
        assert_eq!(stockpile.wood_bonus, 100.0);
        assert_eq!(stockpile.stone_bonus, 100.0);
    }

    #[test]
    fn test_update_resource_caps_system() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn 2 Stockpiles
        world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stockpile { food_bonus: 0.0, wood_bonus: 100.0, stone_bonus: 50.0 },
        ));
        world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stockpile { food_bonus: 0.0, wood_bonus: 100.0, stone_bonus: 50.0 },
        ));

        // Run system
        update_resource_caps_system(&mut world);

        let resources = world.resource::<ColonyResources>();

        // Base (50) + 2 * 100 = 250
        assert_eq!(resources.max_wood, 250.0);
        // Base (20) + 2 * 50 = 120
        assert_eq!(resources.max_stone, 120.0);
    }

    #[test]
    fn test_building_type_stockpile_exists() {
        let bt = BuildingType::Stockpile;
        assert_eq!(bt.label(), "Stockpile");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update ColonyResources

Add fields and helper methods.

```rust
// src/layer1/resources.rs

#[derive(Resource, Debug)]
pub struct ColonyResources {
    pub food: f32,
    pub wood: f32,
    pub stone: f32,

    // New fields
    pub max_food: f32,
    pub max_wood: f32,
    pub max_stone: f32,
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            food: 0.0,
            wood: 0.0,
            stone: 0.0,
            max_food: 50.0,
            max_wood: 50.0,
            max_stone: 20.0,
        }
    }
}

impl ColonyResources {
    pub fn add_wood(&mut self, amount: f32) {
        self.wood = (self.wood + amount).min(self.max_wood);
    }

    pub fn add_stone(&mut self, amount: f32) {
        self.stone = (self.stone + amount).min(self.max_stone);
    }

    pub fn add_food(&mut self, amount: f32) {
        self.food = (self.food + amount).min(self.max_food);
    }
}
```

### 2. Stockpile Component and System

```rust
// src/layer1/stockpile.rs

use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;

#[derive(Component)]
pub struct Stockpile {
    pub food_bonus: f32,
    pub wood_bonus: f32,
    pub stone_bonus: f32,
}

impl Default for Stockpile {
    fn default() -> Self {
        Self {
            food_bonus: 0.0,
            wood_bonus: 100.0,
            stone_bonus: 100.0,
        }
    }
}

pub fn update_resource_caps_system(world: &mut World) {
    let mut total_food_bonus = 0.0;
    let mut total_wood_bonus = 0.0;
    let mut total_stone_bonus = 0.0;

    // Query all stockpiles
    let mut query = world.query::<&Stockpile>();
    for stockpile in query.iter(world) {
        total_food_bonus += stockpile.food_bonus;
        total_wood_bonus += stockpile.wood_bonus;
        total_stone_bonus += stockpile.stone_bonus;
    }

    // Update resources
    let mut resources = world.resource_mut::<ColonyResources>();
    resources.max_food = 50.0 + total_food_bonus; // 50.0 is base constant
    resources.max_wood = 50.0 + total_wood_bonus;
    resources.max_stone = 20.0 + total_stone_bonus;

    // Clamp current resources to new max (if caps reduced)
    resources.food = resources.food.min(resources.max_food);
    resources.wood = resources.wood.min(resources.max_wood);
    resources.stone = resources.stone.min(resources.max_stone);
}
```

### 3. Update BuildingType

```rust
// src/layer1/building.rs

pub enum BuildingType {
    Housing,
    Farm,
    Stockpile, // New variant
}

impl BuildingType {
    pub const fn char(&self) -> char {
        match self {
            Self::Housing => '⌂',
            Self::Farm => '♣',
            Self::Stockpile => '≡', // Stack symbol
        }
    }

    // ... update other methods (color, label, next) ...
}
```

## REFACTOR Phase: Quality & Design

- **Constants**: Extract `BASE_MAX_FOOD`, `BASE_MAX_WOOD`, `BASE_MAX_STONE` to constants.
- **Granary vs Stockpile**: Currently Stockpile handles all (defaults show only wood/stone). Should we add a separate `Granary` building for food?
    - *Decision for now*: `Stockpile` is generic. Can make a `Granary` variant later or just config the bonuses differently.
- **Visuals**: Stockpiles should look different when full? (Too complex for now).

## Acceptance Criteria

- [ ] `ColonyResources` includes `max_*` fields.
- [ ] `add_*` methods clamp values to max.
- [ ] `BuildingType::Stockpile` exists and can be placed.
- [ ] `update_resource_caps_system` correctly calculates totals.
- [ ] Placing a Stockpile increases the UI resource limit.
- [ ] Tests pass with ≥85% coverage.

## Technical Guidance

- **Performance**: Iterating all stockpiles every tick is O(N). If we have 100 stockpiles, it's trivial. Can optimize to `Added<Stockpile>` or `Removed<Stockpile>` queries later if needed.
- **Integration**: Don't forget to register `update_resource_caps_system` in `main.rs`.
- **UI**: Update `render_status_bar` or `render_info_panel` to show `current / max` (e.g., "Wood: 45/150").

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
