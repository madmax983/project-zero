use crate::layer1::items::ItemType;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeCategory {
    Plant,
    Meat,
    Fungi,
    Synthetic, // Nutrient Paste, Rations
    Xeno,      // Alien stuff
}

#[derive(Component, Debug, Clone)]
pub struct GutBiome {
    pub familiarity: HashMap<BiomeCategory, f32>, // 0.0 to 1.0
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
    pub fn get_familiarity(&self, category: BiomeCategory) -> f32 {
        *self.familiarity.get(&category).unwrap_or(&0.0)
    }

    pub fn set_familiarity(&mut self, category: BiomeCategory, value: f32) {
        self.familiarity.insert(category, value.clamp(0.0, 1.0));
    }

    pub fn adapt(&mut self, eaten: BiomeCategory) {
        // Boost eaten
        let current = self.get_familiarity(eaten);
        self.set_familiarity(eaten, current + 0.1); // Fast adaptation

        // Decay others
        for (cat, val) in self.familiarity.iter_mut() {
            if *cat != eaten {
                *val = (*val - 0.01).max(0.0); // Slow decay
            }
        }
    }
}

pub fn get_biome_category(item: &ItemType) -> BiomeCategory {
    match item {
        ItemType::Wheat
        | ItemType::Potato
        | ItemType::Rice
        | ItemType::Corn
        | ItemType::Soy
        | ItemType::Fruit => BiomeCategory::Plant,
        ItemType::Meat | ItemType::Fish => BiomeCategory::Meat,
        ItemType::GlowMushroom => BiomeCategory::Fungi,
        ItemType::MysteryMeal | ItemType::AlienMeatA | ItemType::AlienMeatB => BiomeCategory::Xeno,
        _ => BiomeCategory::Synthetic, // Fallback for processed/rations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::farm::consume_food_system;
    use crate::layer1::morale::Morale;
    use crate::layer1::needs::Needs;
    use crate::layer1::resources::ColonyResources;
    use bevy_ecs::system::RunSystemOnce;

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
        let initial_plant = biome.get_familiarity(BiomeCategory::Plant);

        // Eating Meat should decay Plant familiarity slightly
        biome.adapt(BiomeCategory::Meat);

        assert!(biome.get_familiarity(BiomeCategory::Plant) < initial_plant);
    }

    #[test]
    fn test_digestion_efficiency_impact() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.food = 10.0;
        world.insert_resource(resources);

        // Spawn Farm to supply plant food
        world.spawn((
            crate::layer1::farm::Farm {
                selected_crop: ItemType::Wheat,
                ..Default::default()
            },
            crate::layer1::map::GridPosition { x: 0, y: 0 },
        ));

        // Spawn Pop with POOR Plant adaptation
        let mut poor_biome = GutBiome::default();
        poor_biome.set_familiarity(BiomeCategory::Plant, 0.1);

        let pop = world
            .spawn((
                crate::layer1::pop::Pop,
                Needs {
                    hunger: 0.0,
                    ..Default::default()
                },
                poor_biome,
                Morale::default(),
            ))
            .id();

        // Run eat system
        world.run_system_once(consume_food_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        // Normal gain is e.g. 0.2. Poor adaptation might give 0.1 (0.2 * 0.6 = 0.12).
        assert!(
            needs.hunger < 0.2,
            "Should have reduced nutrition gain due to poor biome"
        );

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.label == "Indigestion"),
            "Should have indigestion debuff"
        );
    }

    #[test]
    fn test_comfort_food_bonus() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.food = 10.0;
        world.insert_resource(resources);

        // Spawn Farm to supply meat food
        // We'll spawn a fauna instead since Farm produces crops
        world.spawn((
            crate::layer1::fauna::Fauna {
                fauna_type: crate::layer1::fauna::FaunaType::SpaceRat,
                ..Default::default()
            },
            crate::layer1::husbandry::Tame::default(),
        ));

        // Spawn Pop with HIGH Meat adaptation
        let mut good_biome = GutBiome::default();
        good_biome.set_familiarity(BiomeCategory::Meat, 1.0);

        let pop = world
            .spawn((
                crate::layer1::pop::Pop,
                Needs {
                    hunger: 0.0,
                    ..Default::default()
                },
                good_biome,
                Morale::default(),
            ))
            .id();

        world.run_system_once(consume_food_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger >= 0.2, "Should have full nutrition gain");

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.label == "Gut Comfort"),
            "Should have comfort buff"
        );
    }
}
