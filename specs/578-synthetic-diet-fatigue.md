# 578: Synthetic Diet Fatigue

## 1. Overview
Late-game "Nutrient Paste" provides perfect caloric and medical sustenance, eliminating starvation entirely. However, Pops exclusively consuming it for extended periods develop "Sensory Deprivation" (Synthetic Diet Fatigue). They stop seeking entertainment or social interaction, their work speed drops, and they eventually become completely catatonic. They require "Real Food" (grown crops or meat) to be shocked back to reality.

## 2. Dependencies
- Core Layer 1 ECS (Entities, Components, Systems)
- Food Consumption system (Pops eating, `FoodType`)
- Work speed and morale modifiers
- `Condition` or `Trait` application

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pops::{Pop, WorkStats, Needs};
    use crate::layer1::food::{FoodType, ConsumeFoodEvent};
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<ConsumeFoodEvent>();
        app.add_systems(Update, process_synthetic_diet_fatigue_system);
        app
    }

    #[test]
    fn test_synthetic_diet_builds_up() {
        let mut app = setup_app();

        // Arrange: A Pop eating Nutrient Paste
        let pop = app.world.spawn((
            Pop,
            SyntheticDietTracker { meals_eaten: 0, ..Default::default() },
        )).id();

        app.world.send_event(ConsumeFoodEvent { pop, food_type: FoodType::NutrientPaste });

        // Act
        app.update();

        // Assert: Tracker increments
        let tracker = app.world.get::<SyntheticDietTracker>(pop).unwrap();
        assert_eq!(tracker.meals_eaten, 1);
    }

    #[test]
    fn test_synthetic_diet_fatigue_applies_sensory_deprivation() {
        let mut app = setup_app();

        // Arrange: A Pop who has eaten too much Nutrient Paste
        let pop = app.world.spawn((
            Pop,
            SyntheticDietTracker { meals_eaten: 10, ..Default::default() }, // 10 meals is the threshold
        )).id();

        app.world.send_event(ConsumeFoodEvent { pop, food_type: FoodType::NutrientPaste });

        // Act
        app.update();

        // Assert: Sensory Deprivation is applied
        assert!(app.world.get::<SensoryDeprivation>(pop).is_some());
    }

    #[test]
    fn test_real_food_cures_sensory_deprivation() {
        let mut app = setup_app();

        // Arrange: A Pop with Sensory Deprivation
        let pop = app.world.spawn((
            Pop,
            SensoryDeprivation,
            SyntheticDietTracker { meals_eaten: 15, ..Default::default() },
        )).id();

        app.world.send_event(ConsumeFoodEvent { pop, food_type: FoodType::RealFood });

        // Act
        app.update();

        // Assert: Deprivation is removed, tracker resets
        assert!(app.world.get::<SensoryDeprivation>(pop).is_none());
        let tracker = app.world.get::<SyntheticDietTracker>(pop).unwrap();
        assert_eq!(tracker.meals_eaten, 0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

pub enum FoodType {
    NutrientPaste,
    RealFood,
}

#[derive(Event)]
pub struct ConsumeFoodEvent {
    pub pop: Entity,
    pub food_type: FoodType,
}

#[derive(Component, Default)]
pub struct SyntheticDietTracker {
    pub meals_eaten: u32,
}

#[derive(Component)]
pub struct SensoryDeprivation;

pub fn process_synthetic_diet_fatigue_system(
    mut events: EventReader<ConsumeFoodEvent>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SyntheticDietTracker)>,
) {
    for event in events.read() {
        if let Ok((entity, mut tracker)) = query.get_mut(event.pop) {
            match event.food_type {
                FoodType::NutrientPaste => {
                    tracker.meals_eaten += 1;
                    if tracker.meals_eaten > 10 {
                        commands.entity(entity).insert(SensoryDeprivation);
                    }
                }
                FoodType::RealFood => {
                    tracker.meals_eaten = 0;
                    commands.entity(entity).remove::<SensoryDeprivation>();
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded `10` for meal threshold. Should extract to a constant like `SENSORY_DEPRIVATION_THRESHOLD`.
- **Performance**: Event iteration is standard.
- **API Improvements**: Map `SensoryDeprivation` to an actual trait or condition component used by the existing systems.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Consuming `NutrientPaste` increments the `SyntheticDietTracker`.
- [ ] Consuming `RealFood` resets the tracker and cures `SensoryDeprivation`.
- [ ] Hitting the threshold adds `SensoryDeprivation` to the Pop.

## 7. Technical Guidance
- Integrate `SensoryDeprivation` into `layer1::execution` to reduce work speed.
- Modify the AI/needs system so that Pops with `SensoryDeprivation` have a `0.0` weight for seeking entertainment.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
