# 087: Food Preservation

## Overview

Currently, food spoils rapidly (`032`), making long-term survival difficult. This spec introduces **Food Preservation** mechanics. Players can build a **Smokehouse** to process raw Food into **Rations**. Rations have a significantly lower spoilage rate than raw food, allowing stockpiling for winter or emergencies.

## Dependencies

- `032` — Entropy & Spoilage (defines spoilage system)
- `008` — Farm (defines food consumption)
- `023` — Refining Industry (defines refining recipe pattern)

## RED Phase: Tests First

Write these tests in `src/layer1/preservation_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::building::BuildingType;
    use crate::layer1::refining::get_refining_recipe;
    use crate::layer1::spoilage::{spoilage_system, GLOBAL_SPOILAGE_RATE};
    use crate::layer1::farm::consume_food_system;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;

    #[test]
    fn test_rations_resource_defaults() {
        let resources = ColonyResources::default();
        assert_eq!(resources.rations, 0.0);
        assert!(resources.max_rations > 0.0);
    }

    #[test]
    fn test_smokehouse_recipe() {
        let resources = ColonyResources {
            food: 5.0,
            wood: 1.0,
            rations: 0.0,
            max_rations: 10.0,
            ..ColonyResources::default()
        };

        // 5 Food + 1 Wood -> 5 Rations
        let (can_refine, input, output) = get_refining_recipe(BuildingType::Smokehouse, &resources);

        assert!(can_refine);
        assert_eq!(input.food, 5.0);
        assert_eq!(input.wood, 1.0);
        assert_eq!(output.rations, 5.0);
    }

    #[test]
    fn test_rations_decay_slower_than_food() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.food = 1000.0;
        resources.rations = 1000.0;
        world.insert_resource(resources);

        spoilage_system(&mut world);

        let res = world.resource::<ColonyResources>();
        let food_loss = 1000.0 - res.food;
        let ration_loss = 1000.0 - res.rations;

        // Rations should decay at 10% the rate of Food
        assert!(ration_loss < food_loss);
        assert!((ration_loss - (food_loss * 0.1)).abs() < 0.001);
    }

    #[test]
    fn test_consume_priority_eats_fresh_food_first() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 1.0, // Just enough for one meal
            rations: 10.0,
            ..Default::default()
        });

        // Spawn hungry pop
        world.spawn((
            Pop,
            Needs { hunger: 0.0, rest: 1.0 }, // Very hungry
        ));

        consume_food_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Should eat the fresh food first
        assert!(res.food < 1.0);
        assert_eq!(res.rations, 10.0);
    }

    #[test]
    fn test_consume_eats_rations_if_no_fresh_food() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            food: 0.0,
            rations: 10.0,
            ..Default::default()
        });

        // Spawn hungry pop
        world.spawn((
            Pop,
            Needs { hunger: 0.0, rest: 1.0 },
        ));

        consume_food_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Should eat rations
        assert!(res.rations < 10.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ColonyResources` (`src/layer1/resources.rs`)

```rust
#[derive(Resource, Default)]
pub struct ColonyResources {
    // ... existing ...
    pub rations: f32,
    pub max_rations: f32, // Default 50.0
}
```

### 2. Update `BuildingType` (`src/layer1/building.rs`)

```rust
pub enum BuildingType {
    // ...
    Smokehouse,
}

// Update impls:
// label() -> "Smokehouse"
// char() -> 'S' (or '♨')
// cost() -> 30 Wood, 10 Stone
```

### 3. Update Refining Recipe (`src/layer1/refining.rs`)

```rust
// Inside get_refining_recipe match:
BuildingType::Smokehouse => (
    res.food >= 5.0 && res.wood >= 1.0 && res.rations < res.max_rations,
    ColonyResources { food: 5.0, wood: 1.0, ..Default::default() },
    ColonyResources { rations: 5.0, ..Default::default() },
),
```

### 4. Update Spoilage System (`src/layer1/spoilage.rs`)

```rust
pub const RATION_SPOILAGE_RATE: f32 = GLOBAL_SPOILAGE_RATE * 0.1;

pub fn spoilage_system(world: &mut World) {
    let mut resources = world.resource_mut::<ColonyResources>();

    // Existing Food decay
    if resources.food > 0.0 {
        let decay = resources.food * GLOBAL_SPOILAGE_RATE;
        resources.food = (resources.food - decay).max(0.0);
    }

    // New Rations decay
    if resources.rations > 0.0 {
        let decay = resources.rations * RATION_SPOILAGE_RATE;
        resources.rations = (resources.rations - decay).max(0.0);
    }

    // ... item decay logic ...
}
```

### 5. Update Consumption System (`src/layer1/farm.rs`)

```rust
pub fn consume_food_system(world: &mut World) {
    let mut resources = world.resource_mut::<ColonyResources>();
    let food_per_meal = 1.0; // Assuming 1.0 for simplicity, verify constant

    // Query hungry pops...
    // For each hungry pop:

    let ate_food = if resources.food >= food_per_meal {
        resources.food -= food_per_meal;
        true
    } else if resources.rations >= food_per_meal {
        resources.rations -= food_per_meal;
        true
    } else {
        false
    };

    if ate_food {
        // Restore hunger logic
    }
}
```

*Note: In `008`, consumption logic iterates pops. Ensure you only deduct resource if available.*

## REFACTOR Phase: Quality & Design

- **Resource Limits**: Ensure `add_rations` respects `max_rations`.
- **UI**: Display `Rations` in the Status Bar (`src/ui/status.rs`).
- **Generic Consumption**: Consider a `Consumable` trait or list for future food types (e.g. Meat, Bread).
- **Feedback**: Add a log when rations spoil? (Only if massive amount).

## Acceptance Criteria

- [ ] `Rations` resource added to `ColonyResources`.
- [ ] `Smokehouse` building added and craftable.
- [ ] Recipe: 5 Food + 1 Wood -> 5 Rations.
- [ ] Rations decay at 10% the rate of Raw Food.
- [ ] Pops prioritize eating Raw Food over Rations.
- [ ] Pops eat Rations if Raw Food is unavailable.
- [ ] Tests pass.

## Technical Guidance

- Be careful with `consume_food_system` loop. If you have 100 pops and 10 food, the first 10 get fed. Ensure you check resource availability *inside* the loop (or decrement a local counter) to avoid over-consumption.
- Use `f32::max(0.0)` for decay to prevent negative values.

## Questions

- Should Rations give different mood? (No, for MVP keep it simple. Survival first).
  - *Architect:* No, all preserved rations should provide the same baseline mood modifier for the MVP.
