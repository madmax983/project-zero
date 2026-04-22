# 1138: Organic Recycling

## 1. Overview

"In the deep void, carbon is carbon. You don't waste atoms."

This specification introduces a new building type, the "Recycler", which converts biological waste (Corpses, Rot, Sewage) into "Nutrient Paste". Consuming Nutrient Paste normally applies a "Gloom" mood modifier to Pops, reflecting the grim reality of their diet. However, Pops with specific traits (e.g., "Pragmatist" or "Cannibal") are immune to this penalty or may even receive a buff.

This creates a tension between basic survival (Food generation) and maintaining humanity (Morale/Diplomacy).

## 2. Dependencies

- `001` — Core Resources (Food, Waste)
- `004` — Pop Entity (Traits, Mood)
- `010` — Building System (Production loops)

## 3. RED Phase: Tests First

```rust
// src/layer1/buildings/recycler_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::core::{ResourceType, Inventory, Pop, Trait, Morale, MoodModifier};
    use crate::layer1::buildings::{Building, ProductionCycle};

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base resources and time
        world
    }

    #[test]
    fn test_recycler_converts_waste_to_paste() {
        let mut world = setup_world();

        let recycler = world.spawn((
            Building { building_type: BuildingType::Recycler },
            Inventory {
                items: vec![(ResourceType::BiologicalWaste, 10)],
                capacity: 50,
            },
            ProductionCycle {
                progress: 0.0,
                duration: 10.0,
                input: vec![(ResourceType::BiologicalWaste, 5)],
                output: vec![(ResourceType::NutrientPaste, 5)],
                active: true,
            }
        )).id();

        // Run production system
        world.run_system_once(crate::layer1::buildings::production_system);

        // Verify output
        let inventory = world.get::<Inventory>(recycler).unwrap();
        assert_eq!(inventory.get_amount(ResourceType::BiologicalWaste), 5);
        assert_eq!(inventory.get_amount(ResourceType::NutrientPaste), 5);
    }

    #[test]
    fn test_eating_paste_causes_gloom() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop { id: 1 },
            Morale { modifiers: vec![] },
            Traits { list: vec![] }
        )).id();

        // Feed pop paste
        world.run_system_once_with((pop, ResourceType::NutrientPaste), crate::layer1::pop::consume_food_system);

        // Verify gloom
        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Ate Nutrient Paste"));
        assert!(morale.modifiers.iter().find(|m| m.label == "Ate Nutrient Paste").unwrap().value < 0.0);
    }

    #[test]
    fn test_pragmatist_ignores_gloom() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop { id: 1 },
            Morale { modifiers: vec![] },
            Traits { list: vec![Trait::Pragmatist] }
        )).id();

        // Feed pop paste
        world.run_system_once_with((pop, ResourceType::NutrientPaste), crate::layer1::pop::consume_food_system);

        // Verify no gloom
        let morale = world.get::<Morale>(pop).unwrap();
        assert!(!morale.modifiers.iter().any(|m| m.label == "Ate Nutrient Paste"));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/buildings/recycler.rs
use bevy_ecs::prelude::*;
use crate::layer1::core::{ResourceType, Inventory, Pop, Trait, Morale, MoodModifier};

pub fn consume_food_system(
    In((pop_entity, food_type)): In<(Entity, ResourceType)>,
    mut query: Query<(&Traits, &mut Morale)>
) {
    if let Ok((traits, mut morale)) = query.get_mut(pop_entity) {
        if food_type == ResourceType::NutrientPaste {
            if !traits.list.contains(&Trait::Pragmatist) && !traits.list.contains(&Trait::Cannibal) {
                if !morale.modifiers.iter().any(|m| m.label == "Ate Nutrient Paste") {
                    morale.modifiers.push(MoodModifier {
                        label: "Ate Nutrient Paste".to_string(),
                        value: -10.0,
                        duration: 100.0,
                    });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- Ensure `ResourceType::BiologicalWaste` and `ResourceType::NutrientPaste` are added to the enum.
- Consider adding a small buff for Cannibals eating Nutrient Paste made from Corpses, though tracking the origin of the paste might require additional state. For MVP, just immunity.
- The Recycler should probably emit an unpleasant aura, affecting the beauty/smell of surrounding tiles.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Recycler building correctly converts waste to paste.
- [ ] Non-pragmatist pops suffer mood penalty when eating paste.

## 7. Technical Guidance

- Use existing `MoodModifier` logic to apply the penalty.
- The exact duration and magnitude of the "Gloom" penalty can be tuned, but should be significant.

## 8. Questions

*Builder: Add any questions here.*
