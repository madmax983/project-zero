# 137: Pop Hobbies

## Overview

Pops are not just machines; they have personalities. When idle or stressed, Pops will engage in **Hobbies** based on their traits. Hobbies provide a way to reduce stress without requiring dedicated buildings (like the Tavern) and produce unique "Curio" items that add flavor to the colony.

## Dependencies

- `016` — Utility AI System (Action selection)
- `084` — Pop Traits (Determines preferred hobby)
- `031` — Pop Morale (Stress mechanics)
- `058` — Personal Tools (Inventory for Curios)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/hobby_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, TraitType};
    use crate::layer1::morale::Stress;
    use crate::layer1::utility_ai::{ActionType, PopEvalData};
    use crate::layer1::hobby::{Hobby, HobbyType, assign_hobby_system, evaluate_hobby};
    use crate::layer1::items::ItemType;
    use crate::layer1::inventory::Inventory;
    use bevy_ecs::prelude::*;

    // 1. Hobby Assignment
    #[test]
    fn test_pop_assigned_hobby_based_on_trait() {
        let mut world = World::new();

        // Spawn pop with Creative trait
        let pop = world.spawn((
            Pop,
            Trait { trait_type: TraitType::Creative, ..Default::default() },
        )).id();

        // Run assignment system
        assign_hobby_system(&mut world);

        // Check hobby
        let hobby = world.get::<Hobby>(pop).unwrap();
        assert_eq!(hobby.hobby_type, HobbyType::Whittling);
    }

    #[test]
    fn test_pop_assigned_random_hobby_if_no_relevant_trait() {
        let mut world = World::new();
        let pop = world.spawn((Pop, Trait::default())).id();

        assign_hobby_system(&mut world);

        let hobby = world.get::<Hobby>(pop);
        assert!(hobby.is_some());
    }

    // 2. Utility Evaluation
    #[test]
    fn test_evaluate_hobby_high_when_stressed() {
        let mut eval_data = PopEvalData::default();
        eval_data.stress = 0.8; // High stress
        eval_data.is_idle = true;

        // Mock hobby component
        let hobby = Hobby { hobby_type: HobbyType::CloudWatching };

        let score = evaluate_hobby(&eval_data, &hobby);
        assert!(score > 0.6, "Stressed pop should want to do hobby");
    }

    #[test]
    fn test_evaluate_hobby_low_when_happy_and_busy() {
        let mut eval_data = PopEvalData::default();
        eval_data.stress = 0.1;
        eval_data.is_idle = false; // Has work

        let hobby = Hobby { hobby_type: HobbyType::CloudWatching };

        let score = evaluate_hobby(&eval_data, &hobby);
        assert!(score < 0.2, "Happy busy pop should not prioritize hobby");
    }

    // 3. Execution & Rewards
    #[test]
    fn test_hobby_execution_reduces_stress() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Stress { value: 0.5, ..Default::default() },
            Hobby { hobby_type: HobbyType::Meditation },
        )).id();

        // Simulate execution tick
        crate::layer1::hobby::execute_hobby_system(&mut world);

        let stress = world.get::<Stress>(pop).unwrap();
        assert!(stress.value < 0.5, "Hobby should reduce stress");
    }

    #[test]
    fn test_hobby_produces_curio() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Hobby { hobby_type: HobbyType::Whittling },
            Inventory::default(),
        )).id();

        // Force production trigger (mocking chance or time)
        crate::layer1::hobby::trigger_hobby_production(&mut world, pop);

        let inventory = world.get::<Inventory>(pop).unwrap();
        let has_curio = inventory.items.iter().any(|i| matches!(i.item_type, ItemType::Curio(_)));
        assert!(has_curio, "Whittling should produce a Curio");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components & Enums

```rust
// src/layer1/hobby.rs
use bevy_ecs::prelude::*;
use crate::layer1::traits::TraitType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HobbyType {
    Whittling,      // Creative
    CloudWatching,  // Lazy/Nature
    Meditation,     // Stoic
    Gossip,         // Social
    Tinkering,      // Industrial
}

#[derive(Component, Debug, Clone)]
pub struct Hobby {
    pub hobby_type: HobbyType,
}
```

### 2. Assignment System

```rust
// src/layer1/hobby.rs
use crate::layer1::traits::Trait;
use rand::Rng;

pub fn assign_hobby_system(mut commands: Commands, query: Query<(Entity, &Trait), Without<Hobby>>) {
    let mut rng = rand::thread_rng();

    for (entity, pop_trait) in &query {
        let hobby_type = match pop_trait.trait_type {
            TraitType::Creative => HobbyType::Whittling,
            TraitType::Lazy => HobbyType::CloudWatching,
            TraitType::Stoic => HobbyType::Meditation,
            TraitType::Social => HobbyType::Gossip,
            TraitType::Industrial => HobbyType::Tinkering,
            _ => {
                // Random default
                match rng.gen_range(0..5) {
                    0 => HobbyType::Whittling,
                    1 => HobbyType::CloudWatching,
                    2 => HobbyType::Meditation,
                    3 => HobbyType::Gossip,
                    _ => HobbyType::Tinkering,
                }
            }
        };

        commands.entity(entity).insert(Hobby { hobby_type });
    }
}
```

### 3. Utility Evaluation

Update `PopEvalData` to include `stress`.

```rust
// src/layer1/hobby.rs

pub fn evaluate_hobby(data: &PopEvalData, _hobby: &Hobby) -> f32 {
    let base_score = 0.1;
    let stress_factor = data.stress * 0.8; // Up to 0.8 from stress
    let idle_bonus = if data.is_idle { 0.3 } else { 0.0 };

    (base_score + stress_factor + idle_bonus).min(1.0)
}
```

### 4. Execution System

```rust
// src/layer1/hobby.rs
use crate::layer1::items::{Item, ItemType};
use crate::layer1::inventory::Inventory;
use crate::layer1::morale::Stress;

pub fn execute_hobby_system(
    mut query: Query<(&mut Stress, &Hobby, Option<&mut Inventory>)>,
    // Filter by ActionType::Hobby (assumed managed by action_system)
) {
    let mut rng = rand::thread_rng();

    for (mut stress, hobby, inventory) in &mut query {
        // Reduce stress
        stress.value = (stress.value - 0.005).max(0.0);

        // Chance to produce item (e.g., 1% per tick)
        if rng.gen_bool(0.01) {
             if let Some(mut inv) = inventory {
                 let item = match hobby.hobby_type {
                     HobbyType::Whittling => Some(Item { item_type: ItemType::Curio("Wooden Duck".to_string()), ..Default::default() }),
                     HobbyType::Tinkering => Some(Item { item_type: ItemType::Curio("Bent Gear".to_string()), ..Default::default() }),
                     _ => None,
                 };

                 if let Some(i) = item {
                     inv.add(i);
                 }
             }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Curio Variety**: Expand `ItemType::Curio` to have more specific subtypes or metadata.
- **Social Hobbies**: `Gossip` should require another Pop nearby (similar to `Socialize` action but informal).
- **Resource Cost**: `Whittling` should consume `Wood` from inventory if available (or spawn "Wood Scraps").
- **Integration**: Add `Hobby` to `ActionType` enum in `utility_ai.rs`.

## Acceptance Criteria

- [ ] `Hobby` component exists and is assigned to Pops.
- [ ] Utility AI evaluates `ActionType::Hobby` based on stress and idle state.
- [ ] Executing hobby reduces `Stress`.
- [ ] Productive hobbies (Whittling, Tinkering) produce `Curio` items.
- [ ] Tests pass.

## Questions

*Builder: How do we handle "Gossip" hobby mechanically?*
*Architect:* It accelerates the spread of Rumors by 50% when the pop interacts with others.
*Architect:* Pops engaged in Gossip form temporary social links and exchange small amounts of their highest/lowest trait values.
