use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::items::ItemType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Category of food for gut biome adaptation.
pub enum BiomeCategory {
    /// Plant-based foods (Wheat, Potato, Rice, etc.).
    Plant,
    /// Animal protein (Meat, Fish).
    Meat,
    /// Fungal foods (`GlowMushroom`).
    Fungi,
    /// Processed or synthetic foods (Nutrient Paste, Rations).
    Synthetic,
    /// Alien flora/fauna (`MysteryMeal`, `AlienMeat`).
    Xeno,
}

#[derive(Component, Debug, Clone)]
/// Component tracking the digestive adaptation of a Pop.
pub struct GutBiome {
    /// Map of familiarity with each food category (0.0 to 1.0).
    pub familiarity: HashMap<BiomeCategory, f32>,
}

impl Default for GutBiome {
    fn default() -> Self {
        let mut map = HashMap::new();
        // Earth standard start
        map.insert(BiomeCategory::Plant, 0.8);
        map.insert(BiomeCategory::Meat, 0.8);
        map.insert(BiomeCategory::Fungi, 0.5);
        map.insert(BiomeCategory::Synthetic, 0.9); // Everyone knows rations
        map.insert(BiomeCategory::Xeno, 0.0);
        Self { familiarity: map }
    }
}

impl GutBiome {
    /// Gets the familiarity for a specific category.
    #[must_use]
    pub fn get_familiarity(&self, category: BiomeCategory) -> f32 {
        *self.familiarity.get(&category).unwrap_or(&0.0)
    }

    /// Sets the familiarity for a specific category.
    pub fn set_familiarity(&mut self, category: BiomeCategory, value: f32) {
        self.familiarity.insert(category, value.clamp(0.0, 1.0));
    }

    /// Adapts the biome to the eaten category (boosts it, decays others).
    pub fn adapt(&mut self, eaten: BiomeCategory) {
        // Boost eaten
        let current = self.get_familiarity(eaten);
        self.set_familiarity(eaten, current + 0.1); // Fast adaptation

        // Decay others
        for (cat, val) in &mut self.familiarity {
            if *cat != eaten {
                *val = (*val - 0.01).max(0.0); // Slow decay
            }
        }
    }
}

/// Helper to map `ItemType` to `BiomeCategory`.
#[must_use]
pub const fn get_biome_category(item: &ItemType) -> BiomeCategory {
    match item {
        ItemType::Wheat | ItemType::Potato | ItemType::Rice | ItemType::Corn | ItemType::Soy | ItemType::Fruit => BiomeCategory::Plant,
        ItemType::Meat | ItemType::Fish => BiomeCategory::Meat,
        ItemType::GlowMushroom => BiomeCategory::Fungi,
        ItemType::MysteryMeal | ItemType::AlienMeatA | ItemType::AlienMeatB => BiomeCategory::Xeno,
        _ => BiomeCategory::Synthetic, // Fallback for processed/rations/tools/etc
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::items::ItemType;
    use crate::layer1::needs::Needs;
    use crate::layer1::morale::Morale;
    use crate::layer1::farm::consume_food_system;
    use crate::layer1::resources::ColonyResources;
    use super::*;

    #[test]
    fn test_gut_biome_initialization() {
        let biome = GutBiome::default();
        // Should start with balanced or specific defaults (e.g. Earth food adapted)
        assert!(biome.get_familiarity(BiomeCategory::Plant) > 0.5);
        assert!(biome.get_familiarity(BiomeCategory::Xeno) < 0.1);
    }

    #[test]
    fn test_adaptation_mechanic() {
        let mut biome = GutBiome::default();
        let initial = biome.get_familiarity(BiomeCategory::Meat);

        // Simulate eating Meat
        biome.adapt(BiomeCategory::Meat);

        assert!(biome.get_familiarity(BiomeCategory::Meat) > initial);
    }

    #[test]
    fn test_decay_mechanic() {
        let mut biome = GutBiome::default();
        biome.set_familiarity(BiomeCategory::Plant, 1.0);
        let initial_plant = biome.get_familiarity(BiomeCategory::Plant);

        // Eating Meat should decay Plant familiarity slightly
        biome.adapt(BiomeCategory::Meat);

        assert!(biome.get_familiarity(BiomeCategory::Plant) < initial_plant);
    }

    #[test]
    fn test_digestion_efficiency_impact() {
        let mut world = World::new();
        // Use zeroed to avoid ambiguity with generic food vs specific crops
        let mut resources = ColonyResources::zeroed();
        resources.food = 10.0;
        resources.wheat = 10.1; // Ensure sum > food so NO generic food is detected
        world.insert_resource(resources);
        world.insert_resource(crate::shared::time::SimulationTime::default());

        // Spawn Pop with POOR Plant adaptation
        let mut poor_biome = GutBiome::default();
        poor_biome.set_familiarity(BiomeCategory::Plant, 0.1);

        let pop = world.spawn((
            crate::layer1::pop::Pop,
            Needs { hunger: 0.0, ..Default::default() },
            poor_biome,
            Morale::default(),
        )).id();

        // Run eat system
        world.run_system_once(consume_food_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        // Normal gain is 0.3. Poor adaptation (< 0.3) gives 0.6x multiplier -> 0.18.
        // 0.18 < 0.2
        assert!(needs.hunger < 0.2, "Should have reduced nutrition gain due to poor biome (expected ~0.18). Actual: {}", needs.hunger);

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Indigestion"), "Should have indigestion debuff");
    }

    #[test]
    fn test_comfort_food_bonus() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.meat = 10.0; // Meat
        world.insert_resource(resources);
        world.insert_resource(crate::shared::time::SimulationTime::default());

        // Spawn Pop with HIGH Meat adaptation
        let mut good_biome = GutBiome::default();
        good_biome.set_familiarity(BiomeCategory::Meat, 1.0);

        let pop = world.spawn((
            crate::layer1::pop::Pop,
            Needs { hunger: 0.0, ..Default::default() },
            good_biome,
            Morale::default(),
        )).id();

        world.run_system_once(consume_food_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger >= 0.2, "Should have full nutrition gain");

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Gut Comfort"), "Should have comfort buff");
    }
}
