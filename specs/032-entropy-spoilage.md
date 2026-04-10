# 032: Entropy and Spoilage

## Overview

Introduce decay mechanics for organic resources to add survival pressure. Food stockpiled in the global inventory should slowly rot, and food items left on the ground should disappear over time. This encourages the player to maintain a steady production of food rather than hoarding it indefinitely without preservation infrastructure (which can be added later).

## Dependencies

- `018` — Mining and Resources (for `ColonyResources` and `ResourceItem`)
- `025` — Hauling Logistics (for `ResourceItem` on ground)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/spoilage_tests.rs (or inside resources.rs tests module)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::spoilage::{Perishable, spoilage_system};

    #[test]
    fn test_perishable_component_default() {
        let p = Perishable::default();
        // Default should be reasonable, e.g., 1000 ticks (1 year)
        assert!(p.max_ticks > 0);
        assert_eq!(p.current_ticks, 0);
    }

    #[test]
    fn test_perishable_item_decay() {
        let mut world = World::new();

        // Spawn a perishable item (Food)
        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Food, amount: 10.0 },
            Perishable { max_ticks: 10, current_ticks: 0 },
        )).id();

        // Run system 5 times
        for _ in 0..5 {
            spoilage_system(&mut world);
        }

        // Item should still exist
        let p = world.get::<Perishable>(item).unwrap();
        assert_eq!(p.current_ticks, 5);
    }

    #[test]
    fn test_perishable_item_destruction() {
        let mut world = World::new();

        // Spawn a perishable item near death
        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Food, amount: 10.0 },
            Perishable { max_ticks: 10, current_ticks: 9 },
        )).id();

        // Run system twice (9->10 (rot), 10->despawn? Or just >= max)
        // Let's say at >= max it is removed.
        spoilage_system(&mut world);

        // Should be gone or marked for removal
        assert!(world.get_entity(item).is_err());
    }

    #[test]
    fn test_global_food_decay() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.food = 1000.0;
        world.insert_resource(resources);

        // Run system
        spoilage_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        // Should be less than 1000.0
        assert!(resources.food < 1000.0);
        // But not zero
        assert!(resources.food > 0.0);
    }

    #[test]
    fn test_non_perishable_items_untouched() {
        let mut world = World::new();

        // Stone is not perishable
        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Stone, amount: 10.0 },
            // No Perishable component
        )).id();

        spoilage_system(&mut world);

        assert!(world.get_entity(item).is_ok());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Perishable Component

```rust
// src/layer1/spoilage.rs

use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;

#[derive(Component, Debug, Clone, Copy)]
pub struct Perishable {
    pub current_ticks: u32,
    pub max_ticks: u32,
}

impl Default for Perishable {
    fn default() -> Self {
        Self {
            current_ticks: 0,
            max_ticks: 1000, // 1 year
        }
    }
}
```

### 2. Implement Spoilage System

```rust
// src/layer1/spoilage.rs

pub const GLOBAL_SPOILAGE_RATE: f32 = 0.0005; // 0.05% per tick

pub fn spoilage_system(world: &mut World) {
    // 1. Handle Global Spoilage
    let mut resources = world.resource_mut::<ColonyResources>();
    if resources.food > 0.0 {
        let decay = resources.food * GLOBAL_SPOILAGE_RATE;
        resources.food = (resources.food - decay).max(0.0);
    }

    // 2. Handle Perishable Items
    let mut to_despawn = Vec::new();
    let mut query = world.query::<(Entity, &mut Perishable)>();

    for (entity, mut perishable) in query.iter_mut(world) {
        perishable.current_ticks += 1;
        if perishable.current_ticks >= perishable.max_ticks {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        world.despawn(entity);
        // Optional: Spawn "Rot" item here later
    }
}
```

### 3. Integrate with Resource Spawning

Modify `mine_rock`, `chop_tree`, and `Farm` harvest logic (wherever `ResourceItem` is spawned) to attach `Perishable` component if the resource is `Food`.

*Note: Since farming/harvesting logic might be scattered, ensure `Perishable` is added during spawn.*

Actually, `ResourceItem` spawning is centralized in:
- `mine_rock` (Stone/Ore - Not perishable)
- `chop_tree` (Wood - Not perishable usually, or long decay. Let's say Wood is non-perishable for MVP).
- `Farm` harvest (Food - Perishable).

Builder must check where `ResourceType::Food` is spawned and add `Perishable`.
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

## REFACTOR Phase: Quality & Design

- **Decay Formula**: The global decay is exponential. Verify if this feels right or if we need linear decay (which requires tracking batches). For MVP, exponential is fine.
- **Seasonality**: In Winter (`027`), spoilage could be slower.
- **Visuals**: Show "Rotting" tooltip on items?
- **Logs**: Log when massive amounts of food rot? (Maybe too spammy).

## Acceptance Criteria

- [ ] `Perishable` component exists.
- [ ] `spoilage_system` runs every tick.
- [ ] Global food decreases over time.
- [ ] Dropped food items disappear after `max_ticks`.
- [ ] Non-food items do not rot.
- [ ] Tests pass.

## Technical Guidance

- Use `TICKS_PER_YEAR` (1000) as a baseline for `max_ticks`.
- When spawning `ResourceItem` in `farm.rs` (or wherever food comes from), verify you add the `Perishable` bundle.
- Ensure `spoilage_system` is added to the app schedule.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
