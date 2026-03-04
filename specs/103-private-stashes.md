# 103: Private Stashes

## Overview

Introduces the concept of **Private Stashes**, where Pops with specific traits (`Greedy`, `Anxious`) steal resources from the global stockpile and hide them in their personal inventory. These resources are removed from `ColonyResources` (making the UI count lower) and are only recovered when the Pop is inspected or the stash is discovered.

This mechanic adds a layer of "inventory unreliability" and gives personality to Pops. `Greedy` pops hoard valuable goods (Metal/Credits/Luxuries), while `Anxious` pops hoard survival goods (Food/Meds).

## Dependencies

- `specs/084-pop-traits.md` (for `Trait` system)
- `specs/022-resource-stockpiles.md` (for `ColonyResources` source)
- `specs/091-the-inspector.md` (for inspection mechanics, or future Justice system)

## RED Phase: Tests First

Write these tests in `src/layer1/private_stash_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use std::collections::{HashMap, HashSet};
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::private_stash::{PrivateStash, hoarding_system, discovery_system, inspect_pop};

    #[test]
    fn test_private_stash_component() {
        let mut stash = PrivateStash::default();
        stash.add(ResourceType::Food, 5.0);
        assert_eq!(stash.get(ResourceType::Food), 5.0);
        assert_eq!(stash.get(ResourceType::Wood), 0.0);
    }

    #[test]
    fn test_hoarding_system_anxious_steals_food() {
        let mut world = World::new();
        // Setup resources
        let mut resources = ColonyResources::default();
        resources.food = 100.0;
        world.insert_resource(resources);

        // Setup Anxious Pop
        let pop = world.spawn((
            Pop,
            Traits(HashSet::from([Trait::Anxious])), // New trait
            PrivateStash::default(),
        )).id();

        // Run system multiple times to ensure probability hits (or mock RNG)
        // For test, we assume system has a high enough chance or we force it.
        // Here we just run it once and check logic logic if we mock RNG,
        // but for integration test, we might loop.
        for _ in 0..100 {
            hoarding_system(&mut world);
        }

        // Check global resources decreased
        let res = world.resource::<ColonyResources>();
        assert!(res.food < 100.0, "Food should be stolen");

        // Check stash increased
        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert!(stash.get(ResourceType::Food) > 0.0, "Stash should contain Food");
    }

    #[test]
    fn test_hoarding_system_greedy_steals_valuables() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.metal = 50.0;
        world.insert_resource(resources);

        let pop = world.spawn((
            Pop,
            Traits(HashSet::from([Trait::Greedy])), // New trait
            PrivateStash::default(),
        )).id();

        for _ in 0..100 {
            hoarding_system(&mut world);
        }

        let res = world.resource::<ColonyResources>();
        assert!(res.metal < 50.0, "Metal should be stolen");

        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert!(stash.get(ResourceType::Metal) > 0.0, "Stash should contain Metal");
    }

    #[test]
    fn test_discovery_returns_resources() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let mut stash = PrivateStash::default();
        stash.add(ResourceType::Food, 10.0);

        let pop = world.spawn((
            Pop,
            stash,
            Traits(HashSet::new()),
        )).id();

        // Inspect the pop
        inspect_pop(&mut world, pop);

        // Stash should be empty
        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert_eq!(stash.get(ResourceType::Food), 0.0);

        // Global resources should have the food back
        let res = world.resource::<ColonyResources>();
        // Default starts at 10.0 + 10.0 recovered = 20.0
        assert_eq!(res.food, 20.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Traits (`src/layer1/traits.rs`)

Add `Greedy` and `Anxious` to the `Trait` enum.

```rust
pub enum Trait {
    // ... existing ...
    Greedy,  // Hoards Valuables (Metal, Luxuries)
    Anxious, // Hoards Survival Goods (Food, Meds)
}
```

### 2. Define Component (`src/layer1/private_stash.rs`)

```rust
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::resources::ResourceType;

#[derive(Component, Debug, Clone, Default)]
pub struct PrivateStash {
    pub inventory: HashMap<ResourceType, f32>,
}

impl PrivateStash {
    pub fn add(&mut self, res: ResourceType, amount: f32) {
        *self.inventory.entry(res).or_insert(0.0) += amount;
    }

    pub fn get(&self, res: ResourceType) -> f32 {
        *self.inventory.get(&res).unwrap_or(&0.0)
    }

    pub fn take_all(&mut self) -> HashMap<ResourceType, f32> {
        std::mem::take(&mut self.inventory)
    }
}
```

### 3. Implement Hoarding System (`src/layer1/private_stash.rs`)

```rust
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::resources::ColonyResources;
use rand::Rng;

pub fn hoarding_system(
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(&Traits, &mut PrivateStash)>,
) {
    let mut rng = rand::thread_rng();

    for (traits, mut stash) in query.iter_mut() {
        // Small chance to steal per tick (e.g., 0.1%)
        if !rng.gen_bool(0.001) { continue; }

        if traits.0.contains(&Trait::Anxious) {
            // Steal Food
            if resources.food >= 1.0 {
                resources.food -= 1.0;
                stash.add(ResourceType::Food, 1.0);
            }
        }

        if traits.0.contains(&Trait::Greedy) {
            // Steal Metal (or Credits/Luxuries in future)
            if resources.metal >= 1.0 {
                resources.metal -= 1.0;
                stash.add(ResourceType::Metal, 1.0);
            }
        }
    }
}
```

### 4. Implement Discovery (`src/layer1/private_stash.rs`)

```rust
pub fn inspect_pop(world: &mut World, pop_entity: Entity) {
    // 1. Get stash content
    let stolen_goods = if let Some(mut stash) = world.get_mut::<PrivateStash>(pop_entity) {
        stash.take_all()
    } else {
        return;
    };

    if stolen_goods.is_empty() { return; }

    // 2. Return to global resources
    let mut resources = world.resource_mut::<ColonyResources>();
    for (res_type, amount) in stolen_goods {
        match res_type {
            ResourceType::Food => resources.add_food(amount),
            ResourceType::Metal => resources.add_metal(amount),
            // ... handle other types ...
            _ => {}
        }
    }

    // 3. Log notification?
    // "Recovered 5 Food from Pop X."
}
```

## REFACTOR Phase: Quality & Design

- **Discovery Mechanic**: Currently `inspect_pop` is a helper. Integrate it with the `Inspector` system (091) or a "Search Room" action.
- **Consequences**: Discovery should cause a morale hit (Shame) or trigger a Justice event (Arrest).
- **Stash Limits**: Stashes shouldn't be infinite. Add a capacity or slow down theft if stash is full.
- **Resource Types**: Expand to `Credits`, `Tools`, `Weapons` as those systems mature.

## Acceptance Criteria

- [ ] `Greedy` and `Anxious` traits added to `Trait` enum.
- [ ] `PrivateStash` component implemented.
- [ ] `hoarding_system` correctly moves resources from global `ColonyResources` to local `PrivateStash`.
- [ ] `inspect_pop` returns resources to global stock.
- [ ] Tests pass.

## Questions

- *Builder: Should pops consume their stash?*
*Architect:* For now, no. They just hoard it. Future iteration could allow them to eat from it if starving.
- *Builder: Does this affect "Total" counts in UI?*
*Architect:* Yes, because `ColonyResources` decreases. The UI shows "Available", not "Total Existing".
