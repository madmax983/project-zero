# 211: Gut Biome

## Overview

Pops are what they eat. The **Gut Biome** simulates the digestive adaptation of colonists to their diet. Eating a specific category of food (e.g., Plant, Meat, Xeno) adapts the gut to digest it efficiently, while neglecting a category causes the ability to digest it to atrophy.

Sudden dietary shifts cause **Indigestion** (Mood penalty, low nutrition), while sticking to a staple diet provides **Comfort** (Mood bonus, high nutrition).

This adds a layer of biological inertia to the food economy. You cannot simply switch from "Wheat" to "Alien Slime" overnight without consequences.

## Dependencies

- `005` — Pop Needs (Hunger mechanics)
- `166` — Xeno-Gastronomy (ItemType classification)
- `034` — Pop Health (Sickness status)

## RED Phase: Tests First

Write these tests in `src/layer1/gut_biome_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::gut_biome::{GutBiome, BiomeCategory, get_biome_category};
    use crate::layer1::items::ItemType;
    use crate::layer1::needs::Needs;
    use crate::layer1::morale::Morale;
    use crate::layer1::farm::consume_food_system;
    use crate::layer1::resources::ColonyResources;

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
        resources.wheat = 10.0; // Plant
        world.insert_resource(resources);

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
        world.run_system_once(consume_food_system);

        let needs = world.get::<Needs>(pop).unwrap();
        // Normal gain is e.g. 0.2. Poor adaptation might give 0.1.
        assert!(needs.hunger < 0.2, "Should have reduced nutrition gain due to poor biome");

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Indigestion"), "Should have indigestion debuff");
    }

    #[test]
    fn test_comfort_food_bonus() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.meat = 10.0; // Meat
        world.insert_resource(resources);

        // Spawn Pop with HIGH Meat adaptation
        let mut good_biome = GutBiome::default();
        good_biome.set_familiarity(BiomeCategory::Meat, 1.0);

        let pop = world.spawn((
            crate::layer1::pop::Pop,
            Needs { hunger: 0.0, ..Default::default() },
            good_biome,
            Morale::default(),
        )).id();

        world.run_system_once(consume_food_system);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger >= 0.2, "Should have full nutrition gain");

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Gut Comfort"), "Should have comfort buff");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Structures

Create `src/layer1/gut_biome.rs`:

```rust
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::items::ItemType;

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
        ItemType::Wheat | ItemType::Potato | ItemType::Rice | ItemType::Corn | ItemType::Soy | ItemType::Fruit => BiomeCategory::Plant,
        ItemType::Meat | ItemType::Fish => BiomeCategory::Meat,
        ItemType::GlowMushroom => BiomeCategory::Fungi,
        ItemType::MysteryMeal | ItemType::AlienMeatA | ItemType::AlienMeatB => BiomeCategory::Xeno,
        _ => BiomeCategory::Synthetic, // Fallback for processed/rations
    }
}
```

### 2. Integrate into Food Consumption

Modify `consume_food_system` in `src/layer1/farm.rs`:

```rust
// In consume_food_system loop:

// ... determine eaten_item ...

if ate {
    // Determine category
    let category = get_biome_category(&eaten_item);

    // Get Biome Data
    let (efficiency, mood_effect) = if let Some(mut biome) = pop_query.get_component_mut::<GutBiome>(entity) {
        let fam = biome.get_familiarity(category);
        biome.adapt(category);

        if fam > 0.8 {
            (1.0, Some("Gut Comfort")) // Bonus
        } else if fam < 0.3 {
            (0.6, Some("Indigestion")) // Penalty
        } else {
            (1.0, None) // Neutral
        }
    } else {
        (1.0, None) // No biome component fallback
    };

    // Apply Hunger with Efficiency
    needs.hunger = (needs.hunger + (HUNGER_PER_MEAL * efficiency)).min(1.0);

    // Apply Mood Effect
    if let Some(label) = mood_effect {
        if let Some(mut morale) = pop_query.get_component_mut::<Morale>(entity) {
            let val = if label == "Gut Comfort" { 0.05 } else { -0.1 };
            morale.add_modifier(label, val, 200);
        }
    }

    // ... existing history/wallet logic ...
}
```

## REFACTOR Phase: Quality & Design

- **Move Consumption Logic**: `consume_food_system` is currently in `farm.rs`. It should probably be in `metabolism.rs` or `gastronomy.rs` to separate "Producing Food" from "Eating Food".
- **Sickness Integration**: Extremely low familiarity (< 0.1) should have a chance to trigger `Health::Sickness` or `Vomiting` (stun).
- **Probiotics**: Add a "Probiotic" item (Medical) that boosts specific biome categories instantly.
- **Traits**: `Iron Stomach` (Ignores indigestion), `Picky Eater` (Faster decay).

## Acceptance Criteria

- [ ] `GutBiome` component added to Pops.
- [ ] `BiomeCategory` correctly maps from `ItemType`.
- [ ] Eating foods adapts the biome (increments familiar, decrements others).
- [ ] Eating unfamiliar food grants reduced hunger and "Indigestion" debuff.
- [ ] Eating familiar food grants "Comfort" buff.
- [ ] All tests pass.

## Technical Guidance

- Ensure `GutBiome` is added to the `Pop` bundle in `spawn_pop`.
- Be careful with `Option<&mut GutBiome>` in queries if you want to support pops without it (e.g. Droids/Aliens later).
- Balance the decay rate so players aren't punished for *reasonable* variety (e.g. rotating Wheat/Potato is fine if both are Plants, but Wheat/Meat rotation needs maintenance).

## Questions

- *Builder: Should "Synthetic" (Rations) cause decay of natural biomes? (Yes, living on rations should atrophy your ability to eat real food).*
    - *Architect: Yes, eating synthetic rations should slowly decrease the gut biome adaptation for natural foods, making the pop sick if they suddenly switch.*
