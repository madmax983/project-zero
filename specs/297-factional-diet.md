# Specification: 297 - The Factional Diet

## 1. Overview
**Layer:** 1
**Fantasy:** Food isn't just sustenance; it's a political statement. You are what your faction eats.
**Mechanic:** Pops belonging to specific Factions develop strong dietary preferences. "Traditionalists" demand Earth-crops; "Transhumanists" prefer optimized Nutrient Paste. Forcing them to eat the opposing diet causes severe Unrest, but feeding them their preferred diet reinforces their Faction loyalty.

## 2. Dependencies
- `068-pop-factions` (Factions and Unrest base systems)
- `114-palette-fatigue` (Food variety and preference systems)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::faction::{Faction, FactionId};
    use crate::layer1::needs::hunger::{DietPreference, FoodItem, FoodType};

    #[test]
    fn test_faction_diet_preferences_assigned_on_join() {
        // Arrange
        let mut world = World::new();
        let pop_entity = world.spawn(PopBundle::default()).id();
        let traditionalist_faction = FactionId::new("Traditionalists");

        // Act
        world.entity_mut(pop_entity).insert(FactionMember { faction: traditionalist_faction });

        // Assert: Pop should now have a strong preference for EarthCrops
        let prefs = world.get::<DietPreference>(pop_entity).unwrap();
        assert_eq!(prefs.preferred_type, FoodType::EarthCrop);
        assert_eq!(prefs.hated_type, FoodType::NutrientPaste);
    }

    #[test]
    fn test_eating_hated_diet_causes_unrest() {
        // Arrange
        let mut world = World::new();
        let pop_entity = world.spawn((
            PopBundle::default(),
            FactionMember { faction: FactionId::new("Traditionalists") },
            DietPreference { preferred_type: FoodType::EarthCrop, hated_type: FoodType::NutrientPaste }
        )).id();

        // Act
        let food = FoodItem { food_type: FoodType::NutrientPaste, nutrition: 10.0 };
        process_food_consumption(&mut world, pop_entity, food);

        // Assert: Eating hated food should generate Unrest and Stress
        let stress = world.get::<StressTracker>(pop_entity).unwrap();
        let unrest = world.get::<UnrestFactor>(pop_entity).unwrap();
        assert!(stress.accumulated_stress > 5.0, "Should generate significant stress");
        assert!(unrest.value > 0.0, "Should generate political unrest");
    }

    #[test]
    fn test_eating_preferred_diet_boosts_loyalty() {
        // Arrange
        let mut world = World::new();
        let pop_entity = world.spawn((
            PopBundle::default(),
            FactionMember { faction: FactionId::new("Transhumanists") },
            FactionLoyalty { value: 50.0 },
            DietPreference { preferred_type: FoodType::NutrientPaste, hated_type: FoodType::EarthCrop }
        )).id();

        // Act
        let food = FoodItem { food_type: FoodType::NutrientPaste, nutrition: 10.0 };
        process_food_consumption(&mut world, pop_entity, food);

        // Assert: Eating preferred food should boost faction loyalty
        let loyalty = world.get::<FactionLoyalty>(pop_entity).unwrap();
        assert!(loyalty.value > 50.0, "Should increase faction loyalty");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/social/faction_diet.rs

use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::social::faction::{FactionMember, FactionLoyalty, UnrestFactor};
use crate::layer1::needs::hunger::{FoodItem, FoodType, StressTracker};

#[derive(Component, Debug, Clone)]
pub struct DietPreference {
    pub preferred_type: FoodType,
    pub hated_type: FoodType,
}

pub fn assign_faction_diets(
    mut commands: Commands,
    query: Query<(Entity, &FactionMember), Without<DietPreference>>
) {
    for (entity, faction_member) in query.iter() {
        let prefs = match faction_member.faction.as_str() {
            "Traditionalists" => DietPreference { preferred_type: FoodType::EarthCrop, hated_type: FoodType::NutrientPaste },
            "Transhumanists" => DietPreference { preferred_type: FoodType::NutrientPaste, hated_type: FoodType::EarthCrop },
            _ => DietPreference { preferred_type: FoodType::Any, hated_type: FoodType::None },
        };
        commands.entity(entity).insert(prefs);
    }
}

pub fn process_food_consumption(
    world: &mut World,
    pop_entity: Entity,
    food: FoodItem
) {
    let mut stress = 0.0;
    let mut unrest = 0.0;
    let mut loyalty_boost = 0.0;

    if let Some(prefs) = world.get::<DietPreference>(pop_entity) {
        if food.food_type == prefs.hated_type {
            stress += 10.0;
            unrest += 5.0;
        } else if food.food_type == prefs.preferred_type {
            loyalty_boost += 2.0;
        }
    }

    if let mut stress_tracker = world.get_mut::<StressTracker>(pop_entity) {
        stress_tracker.accumulated_stress += stress;
    }
    if let mut unrest_factor = world.get_mut::<UnrestFactor>(pop_entity) {
        unrest_factor.value += unrest;
    }
    if let mut faction_loyalty = world.get_mut::<FactionLoyalty>(pop_entity) {
        faction_loyalty.value += loyalty_boost;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The `process_food_consumption` function should be integrated into the main `eat_system` or dispatched via Bevy events to decouple the logic.
- **Configurable Factions**: Diet preferences should not be hardcoded by string matching in `assign_faction_diets`. Instead, they should be loaded from a `FactionConfig` resource.
- **Modifiers**: Consider adding a temporary `FoodSatisfaction` component to track the duration of the buff/debuff instead of just an instantaneous flat increase.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Eating opposite faction foods correctly applies Unrest and Stress.
- [ ] Eating preferred faction foods correctly buffs Loyalty.

## 7. Technical Guidance
- Ensure that the new `DietPreference` component handles `FoodType::Any` gracefully without applying unwarranted buffs or debuffs.
- When applying Unrest, make sure to clamp it to valid ranges if the base system requires it.
- Use `#[cfg(test)]` to keep tests co-located but not compiled into the final binary.

## 8. Questions
*Builder: add questions here if spec is unclear.*
