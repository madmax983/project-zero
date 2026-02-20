//! Xeno-Gastronomy system (Spec 166).
//!
//! Handles the creation of "Mystery Meals" from alien ingredients, with randomized effects.

use bevy_ecs::prelude::*;
use crate::layer1::items::ItemType;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::hash::{Hash, Hasher};

/// Possible effects of consuming a Mystery Meal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MealEffect {
    /// No effect.
    None,
    /// Increases Morale.
    MoodBoost,
    /// Decreases Morale.
    MoodDebuff,
    /// Increases work speed.
    HighEnergy,
    /// Decreases work speed.
    Lethargy,
    /// Damages health.
    Poison,
    /// Causes visual glitches (client-side only).
    Hallucination,
}

/// Tracks discovered recipes (Ingredient -> Effect).
#[derive(Resource, Default)]
pub struct Cookbook {
    /// Known associations between ingredients and effects.
    pub known_recipes: std::collections::HashMap<ItemType, MealEffect>,
}

impl Cookbook {
    /// Records a discovery.
    pub fn discover(&mut self, ingredient: ItemType, effect: MealEffect) {
        self.known_recipes.insert(ingredient, effect);
    }

    /// Checks if an ingredient's effect is known.
    #[must_use]
    pub fn is_known(&self, ingredient: &ItemType) -> bool {
        self.known_recipes.contains_key(ingredient)
    }

    /// Gets the known effect of an ingredient.
    #[must_use]
    pub fn get_effect(&self, ingredient: &ItemType) -> Option<MealEffect> {
        self.known_recipes.get(ingredient).copied()
    }
}

/// Component storing the hidden effect of a cooked meal.
#[derive(Component)]
pub struct MealHiddenEffect {
    /// The effect that will trigger when eaten.
    pub effect: MealEffect,
}

/// Generates a deterministic effect for an item type based on a seed.
#[must_use]
pub fn generate_meal_effect(seed: u64, item: &ItemType) -> MealEffect {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
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

/// Performs a cooking experiment, consuming an ingredient to create a Mystery Meal.
pub fn perform_experiment(world: &mut World, _chef: Entity, ingredient_entity: Entity) -> bool {
    // 1. Get Ingredient Type
    let item_type = if let Some(item) = world.get::<crate::layer1::items::Item>(ingredient_entity) {
        item.item_type.clone()
    } else {
        return false;
    };

    // 2. Consume Ingredient
    world.despawn(ingredient_entity);

    // 3. Determine Effect (using hardcoded seed 12345 for MVP)
    let effect = generate_meal_effect(12345, &item_type);

    // 4. Produce Mystery Meal with Effect Component
    world.spawn((
        crate::layer1::items::Item { item_type: crate::layer1::items::ItemType::MysteryMeal },
        MealHiddenEffect { effect },
        crate::layer1::map::GridPosition { x: 0, y: 0 } // Should be at Chef's pos ideally
    ));

    // 5. Update Cookbook
    // For GREEN phase, we assume the chef learns immediately upon cooking.
    if let Some(mut cookbook) = world.get_resource_mut::<Cookbook>() {
        cookbook.discover(item_type, effect);
    }

    true
}

/// Applies the meal effect to a pop.
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
        MealEffect::MoodDebuff => {
             if let Some(mut morale) = world.get_mut::<crate::layer1::morale::Morale>(pop) {
                morale.modifiers.push(crate::layer1::morale::MoodModifier {
                    label: "Disgusting Meal".to_string(),
                    value: -0.2,
                    duration: 100,
                });
            }
        },
        // TODO: Implement other effects
        _ => {}
    }
}

/// System for analyzing ingredients (placeholder).
pub const fn analyze_ingredient_system() {}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::gastronomy::{
        Cookbook, MealEffect,
        generate_meal_effect
    };
    use crate::layer1::pop::Pop;
    use crate::layer1::morale::Morale;

    #[test]
    fn test_generate_consistent_effect() {
        // Effect should be deterministic based on seed + item type
        let effect1 = generate_meal_effect(12345, &ItemType::AlienMeatA);
        let effect2 = generate_meal_effect(12345, &ItemType::AlienMeatA);

        assert_eq!(effect1, effect2, "Same seed/item should yield same effect");

        // Try to find a seed that yields a different effect
        let mut found_different = false;
        for i in 0..100 {
            let effect3 = generate_meal_effect(12346 + i, &ItemType::AlienMeatA);
            if effect1 != effect3 {
                found_different = true;
                break;
            }
        }
        assert!(found_different, "Different seed should eventually yield different effect");
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
        world.insert_resource(Cookbook::default()); // Need resource to record

        // Setup Chef and Ingredients
        let chef = world.spawn(Pop).id();
        let ingredient = world.spawn(Item {
            item_type: ItemType::AlienMeatA,
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
