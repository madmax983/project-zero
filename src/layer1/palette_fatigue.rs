use crate::layer1::items::ItemType;
use crate::layer1::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;
use std::collections::VecDeque;

/// Tracks the last 5 meals eaten by a Pop to calculate palette fatigue.
#[derive(Component, Debug, Clone, Default)]
pub struct DietaryHistory {
    /// The most recent meals.
    pub recent_meals: VecDeque<ItemType>,
}

/// Records a meal in the history, maintaining a max size of 5.
pub fn record_meal(history: &mut DietaryHistory, item: ItemType) {
    history.recent_meals.push_back(item);
    if history.recent_meals.len() > 5 {
        history.recent_meals.pop_front();
    }
}

/// Calculates palette fatigue score (0.0 to 1.0).
/// 0.0 = Varied diet.
/// 1.0 = Complete monoculture (all same).
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_palette_fatigue(history: &DietaryHistory) -> f32 {
    let total = history.recent_meals.len() as f32;
    if total == 0.0 {
        return 0.0;
    }

    // Count repetitions
    let mut counts = bevy::utils::HashMap::new();
    for item in &history.recent_meals {
        *counts.entry(item).or_insert(0) += 1;
    }

    // Max repetition count
    let max_rep = *counts.values().max().unwrap_or(&0) as f32;
    let ratio = max_rep / total;

    // Baseline is 1/total (perfect variety).
    let baseline = 1.0 / total;

    // Avoid division by zero if total is 1 (baseline is 1.0)
    if baseline >= 1.0 {
        return 0.0;
    }

    if ratio <= baseline {
        return 0.0;
    }

    // Normalize range [baseline, 1.0] to [0.0, 1.0]
    (ratio - baseline) / (1.0 - baseline)
}

/// System to apply morale modifiers based on dietary variety.
pub fn apply_palette_fatigue_system(mut query: Query<(&mut Morale, &DietaryHistory)>) {
    for (mut morale, history) in &mut query {
        let fatigue = calculate_palette_fatigue(history);

        // Remove old modifier
        morale
            .modifiers
            .retain(|m| m.label != "Boring Diet" && m.label != "Varied Diet");

        if fatigue >= 0.75 {
            // High fatigue
            morale.modifiers.push(MoodModifier {
                label: "Boring Diet".to_string(),
                value: -0.1 * fatigue, // Scales with fatigue
                duration: 200,         // Lingers
            });
        } else if fatigue <= 0.25 && history.recent_meals.len() >= 3 {
            morale.modifiers.push(MoodModifier {
                label: "Varied Diet".to_string(),
                value: 0.05,
                duration: 200,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::items::ItemType;
    use crate::layer1::morale::Morale;
    use crate::layer1::palette_fatigue::{calculate_palette_fatigue, record_meal, DietaryHistory};
    use crate::layer1::pop::Pop;
    use bevy_ecs::prelude::*;

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
        // 5 Potatoes = 1.0 ratio. Baseline 0.2. Normalized 1.0.
        let fatigue = calculate_palette_fatigue(&history);
        assert!(fatigue > 0.5);
    }

    #[test]
    fn test_fatigue_calculation_varied() {
        let mut history = DietaryHistory::default();
        record_meal(&mut history, ItemType::Potato);
        record_meal(&mut history, ItemType::Wheat);
        record_meal(&mut history, ItemType::Meat);
        record_meal(&mut history, ItemType::Fish);
        record_meal(&mut history, ItemType::Fruit);

        // 1 of each. Ratio 0.2. Baseline 0.2. Result 0.0.
        let fatigue = calculate_palette_fatigue(&history);
        assert_eq!(fatigue, 0.0);
    }

    #[test]
    fn test_fatigue_calculation_four_same_triggers_penalty() {
        let mut history = DietaryHistory::default();
        // 4 Potatoes, 1 Wheat
        for _ in 0..4 {
            record_meal(&mut history, ItemType::Potato);
        }
        record_meal(&mut history, ItemType::Wheat);

        // Fatigue check
        // Ratio = 0.8. Baseline = 0.2.
        // Normalized = (0.8 - 0.2) / 0.8 = 0.75.
        // Current threshold is >= 0.8. So this test will pass fatigue calculation but fail system application if system expects penalty.

        // Run system
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                Morale {
                    value: 0.8,
                    modifiers: Vec::new(),
                },
                history,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(super::apply_palette_fatigue_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        // Should have "Boring Diet" modifier
        let modifier = morale.modifiers.iter().find(|m| m.label == "Boring Diet");
        assert!(
            modifier.is_some(),
            "Should have Boring Diet modifier for 4/5 same meals"
        );
    }

    #[test]
    fn test_apply_morale_modifier() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                Morale {
                    value: 0.8,
                    modifiers: Vec::new(),
                },
                DietaryHistory {
                    recent_meals: vec![ItemType::Potato; 5].into(),
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(super::apply_palette_fatigue_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        // Check for "Boring Diet" modifier
        let modifier = morale.modifiers.iter().find(|m| m.label == "Boring Diet");
        assert!(modifier.is_some());
        if let Some(m) = modifier {
            assert!(m.value < 0.0);
        }
    }
}
