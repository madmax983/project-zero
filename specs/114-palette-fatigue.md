# 114: Palette Fatigue

## Overview

Pops get bored of eating the same food item repeatedly. This mechanic encourages players to diversify food production rather than relying on a monoculture (e.g., only Potatoes). A "Boring Diet" causes morale penalties, while a "Varied Diet" provides bonuses.

## Dependencies

- `005` — Pop Needs (Hunger)
- `009` — Stockpiles (Food Items)
- `031` — Pop Morale (Morale Component)

## RED Phase: Tests First

Write these tests in `src/layer1/palette_fatigue_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::items::ItemType;
    use crate::layer1::morale::{Morale, MoraleModifier};
    use crate::layer1::palette_fatigue::{DietaryHistory, record_meal, calculate_palette_fatigue};

    #[test]
    fn test_record_meal_updates_history() {
        let mut history = DietaryHistory::default();
        record_meal(&mut history, ItemType::Potato);
        assert_eq!(history.recent_meals.len(), 1);
        assert_eq!(history.recent_meals[0], ItemType::Potato);

        record_meal(&mut history, ItemType::Wheat);
        assert_eq!(history.recent_meals.len(), 2);
        assert_eq!(history.recent_meals[1], ItemType::Wheat);
    }

    #[test]
    fn test_history_limit() {
        let mut history = DietaryHistory::default();
        // Max history length is 5
        for _ in 0..10 {
            record_meal(&mut history, ItemType::Potato);
        }
        assert_eq!(history.recent_meals.len(), 5);
    }

    #[test]
    fn test_fatigue_calculation_monoculture() {
        let mut history = DietaryHistory::default();
        for _ in 0..5 {
            record_meal(&mut history, ItemType::Potato);
        }
        // 5 Potatoes = High Fatigue
        let fatigue = calculate_palette_fatigue(&history);
        assert!(fatigue > 0.5); // Normalized 0.0-1.0
    }

    #[test]
    fn test_fatigue_calculation_varied() {
        let mut history = DietaryHistory::default();
        record_meal(&mut history, ItemType::Potato);
        record_meal(&mut history, ItemType::Wheat);
        record_meal(&mut history, ItemType::Meat);
        record_meal(&mut history, ItemType::Fish);
        record_meal(&mut history, ItemType::Fruit);

        let fatigue = calculate_palette_fatigue(&history);
        assert_eq!(fatigue, 0.0);
    }

    #[test]
    fn test_apply_morale_modifier() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Morale::default(),
            DietaryHistory {
                recent_meals: vec![ItemType::Potato; 5].into(),
            },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::palette_fatigue::apply_palette_fatigue_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        // Check for "Boring Diet" modifier
        let modifier = morale.modifiers.iter().find(|m| m.label == "Boring Diet");
        assert!(modifier.is_some());
        assert!(modifier.unwrap().value < 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Component (`src/layer1/palette_fatigue.rs`)

```rust
use bevy_ecs::prelude::*;
use std::collections::VecDeque;
use crate::layer1::items::ItemType;
use crate::layer1::morale::{Morale, MoraleModifier};

#[derive(Component, Debug, Clone, Default)]
pub struct DietaryHistory {
    pub recent_meals: VecDeque<ItemType>,
}

pub fn record_meal(history: &mut DietaryHistory, item: ItemType) {
    history.recent_meals.push_back(item);
    if history.recent_meals.len() > 5 {
        history.recent_meals.pop_front();
    }
}

pub fn calculate_palette_fatigue(history: &DietaryHistory) -> f32 {
    if history.recent_meals.is_empty() {
        return 0.0;
    }

    // Count repetitions
    let mut counts = std::collections::HashMap::new();
    for item in &history.recent_meals {
        *counts.entry(item).or_insert(0) += 1;
    }

    // Max repetition count / Total meals
    let max_rep = counts.values().max().unwrap_or(&0);
    let total = history.recent_meals.len() as f32;

    // If 5/5 are same, fatigue = 1.0. If 1/5, fatigue = 0.2 (baseline).
    // Normalize: (Repetition Ratio - 0.2) / 0.8?
    // Simplified: Just use raw ratio.
    (*max_rep as f32 / total)
}

pub fn apply_palette_fatigue_system(
    mut query: Query<(&mut Morale, &DietaryHistory)>,
) {
    for (mut morale, history) in query.iter_mut() {
        let fatigue = calculate_palette_fatigue(history);

        // Remove old modifier
        morale.remove_modifier("Boring Diet");
        morale.remove_modifier("Varied Diet");

        if fatigue >= 0.8 { // 4 or 5 out of 5 same
            morale.add_modifier(MoraleModifier {
                label: "Boring Diet".to_string(),
                value: -0.1 * (fatigue),
                duration: 200, // Lingers
                source: "Diet".to_string(),
            });
        } else if fatigue <= 0.4 && history.recent_meals.len() >= 3 {
            morale.add_modifier(MoraleModifier {
                label: "Varied Diet".to_string(),
                value: 0.05,
                duration: 200,
                source: "Diet".to_string(),
            });
        }
    }
}
```

### 2. Hook into `eat_action`

In `src/layer1/actions.rs` (or where eating happens):

```rust
// Inside perform_eat_action
if let Some(mut history) = world.get_mut::<DietaryHistory>(entity) {
    record_meal(&mut history, item_type);
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Use a fixed-size array (`[ItemType; 5]`) instead of `VecDeque` to avoid heap allocation if possible, or use a ring buffer.
- **Config**: Make history length and fatigue thresholds distinct constants.
- **UI**: Display "Recent Meals" in the Pop Inspector UI to explain the mood penalty.
- **Integration**: Ensure `DietaryHistory` is added to new Pops on spawn.

## Acceptance Criteria

- [ ] `DietaryHistory` component tracks last 5 meals.
- [ ] Eating the same item 4+ times in a row causes a negative Morale modifier.
- [ ] Eating different items causes a positive Morale modifier (or at least no penalty).
- [ ] Tests pass with >85% coverage.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
