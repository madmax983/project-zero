# 166: Xeno-Gastronomy

## Overview

Introduces the **Chef** job and specialized cooking at the **Tavern** (or Kitchen). Chefs can use alien ingredients (e.g., from `Modular Fauna` drops, `Xeno-Botany`, or `Genetic Samples`) to create "Mystery Meals".

These meals have randomized effects (Buffs, Debuffs, Poison, Hallucinations) determined by the Ingredient Type + World Seed. Successful experiments are recorded in a **Cookbook**, allowing reproducible recipes.

This adds a layer of discovery and risk to the food system, moving beyond simple "Hunger" reduction to "Mood" and "Health" manipulation.

## Dependencies

- `097` — Social Tavern (Base building)
- `075` — Animal Husbandry (Meat source)
- `032` — Entropy & Spoilage (Food items)
- `120` — Crop Diversity (Vegetable source, optional but recommended)
- `164` — Modular Fauna (Alien meat source)

## RED Phase: Tests First

Write these tests in `src/layer1/gastronomy_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::gastronomy::{
        Cookbook, CookingExperiment, MealEffect, analyze_ingredient_system,
        generate_meal_effect
    };
    use crate::layer1::pop::Pop;
    use crate::layer1::morale::Morale;

    #[test]
    fn test_generate_consistent_effect() {
        // Effect should be deterministic based on seed + item type
        let effect1 = generate_meal_effect(12345, ItemType::AlienMeatA);
        let effect2 = generate_meal_effect(12345, ItemType::AlienMeatA);
        let effect3 = generate_meal_effect(67890, ItemType::AlienMeatA);

        assert_eq!(effect1, effect2, "Same seed/item should yield same effect");
        assert_ne!(effect1, effect3, "Different seed should yield different effect");
    }

    #[test]
    fn test_cookbook_recording() {
        let mut world = World::new();
        world.insert_resource(Cookbook::default());

        // Record a discovery
        let mut cookbook = world.resource_mut::<Cookbook>();
        cookbook.discover(ItemType::AlienMeatA, MealEffect::HighEnergy);

        assert!(cookbook.is_known(&ItemType::AlienMeatA));
        assert_eq!(cookbook.get_effect(&ItemType::AlienMeatA), Some(MealEffect::HighEnergy));
    }

    #[test]
    fn test_cooking_experiment_consumes_ingredients() {
        let mut world = World::new();
        // Setup Chef and Ingredients
        let chef = world.spawn(Pop).id();
        let ingredient = world.spawn(Item {
            item_type: ItemType::AlienMeatA,
            ..Default::default()
        }).id();

        // Add Cooking Experiment component to Chef or Workstation
        // For simplicity, let's say Chef performs action
        let success = crate::layer1::gastronomy::perform_experiment(&mut world, chef, ingredient);

        assert!(success);
        assert!(world.get_entity(ingredient).is_err(), "Ingredient should be consumed");

        // Check for Output Meal
        let mut query = world.query::<&Item>();
        let found = query.iter(&world).any(|i| i.item_type == ItemType::MysteryMeal);
        assert!(found, "Should produce Mystery Meal");
    }

    #[test]
    fn test_eating_mystery_meal_applies_effect() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Morale { value: 0.5, ..Default::default() }
        )).id();

        // Apply effect manually (simulating eating)
        let effect = MealEffect::MoodBoost;
        crate::layer1::gastronomy::apply_meal_effect(&mut world, pop, effect);

        let morale = world.get::<Morale>(pop).unwrap();
        // Assuming MoodBoost adds a modifier
        assert!(morale.modifiers.iter().any(|m| m.label == "Delicious Meal"));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Enums

- **ItemType**: Add `AlienMeatA`, `AlienMeatB`, `GlowMushroom`, `MysteryMeal`.
- **JobType**: Add `Chef`.

### 2. Define Structures

```rust
// src/layer1/gastronomy.rs

use bevy_ecs::prelude::*;
use crate::layer1::items::ItemType;
use rand::{Rng, SeedableRng, rngs::StdRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MealEffect {
    None,
    MoodBoost,    // +Morale
    MoodDebuff,   // -Morale
    HighEnergy,   // +Work Speed
    Lethargy,     // -Work Speed
    Poison,       // Health damage
    Hallucination,// Visual effects (handled by UI)
}

#[derive(Resource, Default)]
pub struct Cookbook {
    pub known_recipes: std::collections::HashMap<ItemType, MealEffect>,
}

impl Cookbook {
    pub fn discover(&mut self, ingredient: ItemType, effect: MealEffect) {
        self.known_recipes.insert(ingredient, effect);
    }

    pub fn is_known(&self, ingredient: &ItemType) -> bool {
        self.known_recipes.contains_key(ingredient)
    }

    pub fn get_effect(&self, ingredient: &ItemType) -> Option<MealEffect> {
        self.known_recipes.get(ingredient).cloned()
    }
}

pub fn generate_meal_effect(seed: u64, item: ItemType) -> MealEffect {
    // Hash item type + seed to deterministic random
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    item.hash(&mut hasher);
    let item_hash = hasher.finish();

    let combined_seed = seed.wrapping_add(item_hash);
    let mut rng = StdRng::seed_from_u64(combined_seed);

    match rng.gen_range(0..6) {
        0 => MealEffect::MoodBoost,
        1 => MealEffect::MoodDebuff,
        2 => MealEffect::HighEnergy,
        3 => MealEffect::Lethargy,
        4 => MealEffect::Poison,
        _ => MealEffect::None,
    }
}

pub fn perform_experiment(world: &mut World, _chef: Entity, ingredient_entity: Entity) -> bool {
    // 1. Get Ingredient Type
    let item_type = if let Ok(item) = world.get::<crate::layer1::items::Item>(ingredient_entity) {
        item.item_type
    } else {
        return false;
    };

    // 2. Consume Ingredient
    world.despawn(ingredient_entity);

    // 3. Determine Effect (using World Seed resource, assume 12345 for now)
    let effect = generate_meal_effect(12345, item_type);

    // 4. Produce Mystery Meal with Effect Component
    // We need a component to store the hidden effect on the item
    world.spawn((
        crate::layer1::items::Item { item_type: crate::layer1::items::ItemType::MysteryMeal },
        MealHiddenEffect { effect },
        crate::layer1::map::GridPosition { x: 0, y: 0 } // Should be at Chef's pos
    ));

    // 5. Update Cookbook? Only after eating!
    // Or maybe Chef "tastes" it? Let's say Chef tastes it immediately for Green phase.
    let mut cookbook = world.resource_mut::<Cookbook>();
    cookbook.discover(item_type, effect);

    true
}

#[derive(Component)]
pub struct MealHiddenEffect {
    pub effect: MealEffect,
}

pub fn apply_meal_effect(world: &mut World, pop: Entity, effect: MealEffect) {
    match effect {
        MealEffect::MoodBoost => {
            if let Some(mut morale) = world.get_mut::<crate::layer1::morale::Morale>(pop) {
                morale.modifiers.push(crate::layer1::morale::MoodModifier {
                    label: "Delicious Meal".to_string(),
                    value: 0.2,
                    duration: 100,
                });
            }
        },
        MealEffect::Poison => {
             // Damage health
        },
        _ => {}
    }
}
```

## REFACTOR Phase: Quality & Design

- **UI Integration**: `Cookbook` resource needs a UI panel to view unlocked recipes.
- **Tasting Logic**: Separate "Cooking" from "Tasting". The Chef cooks a `MysteryMeal`. When a Pop eats it, the effect triggers and the recipe is revealed to the colony (via `Notifications`).
- **Skill Check**: Higher `Gastronomy` skill increases chance of *positive* mutations or removing negative traits from ingredients (e.g., removing `Poison` from `Pufferfish`).
- **Ingredients**: Define `Ingredient` component/tag to filter what can be cooked.

## Acceptance Criteria

- [ ] `Chef` job exists and Pops can be assigned.
- [ ] `Alien` ingredients exist as items.
- [ ] Cooking an unknown ingredient produces a `MysteryMeal`.
- [ ] Eating the meal triggers a randomized effect (consistent per save).
- [ ] Cookbook records the effect after consumption.
- [ ] Tests pass.

## Technical Guidance

- Use `ItemType::MysteryMeal` as a generic container. Use `Component` to store the specific `MealEffect`.
- Ensure `generate_meal_effect` uses the `WorldSeed` resource, not a hardcoded value.
- Hook into `eat_system` to check for `MealHiddenEffect` component.

## Questions

- *Builder: Should "Mystery Meals" stack?*
- *Architect: Mystery Meals stack only by exact ingredient.*
