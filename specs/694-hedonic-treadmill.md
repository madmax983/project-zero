# 694: The Hedonic Treadmill

## Overview

A feature where Pops track the quality of the items they consume (e.g., Food, Beds, Clothing). After consuming high-quality items for a sustained period, their "Standard of Living" increases. Reverting to lower-quality items causes a massive Mood penalty, worse than if they had never experienced the luxury. This creates tension between boosting morale immediately and maintaining a sustainable baseline expectation.

## Dependencies

- `005` — Pop Needs
- `031` — Pop Morale

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_consuming_high_quality_increases_standard() {
        let mut app = App::new();

        let pop_id = app.world_mut().spawn((
            Pop::default(),
            StandardOfLiving { current_level: 1.0, quality_history: vec![] },
        )).id();

        // Consume a high-quality item multiple times
        for _ in 0..10 {
            app.world_mut().send_event(ConsumeItemEvent {
                consumer: pop_id,
                item_quality: 5.0,
            });
            app.update();
        }

        // Standard of Living should have increased toward 5.0
        let sol = app.world().get::<StandardOfLiving>(pop_id).unwrap();
        assert!(sol.current_level > 1.0);
    }

    #[test]
    fn test_consuming_below_standard_causes_mood_penalty() {
        let mut app = App::new();

        let pop_id = app.world_mut().spawn((
            Pop::default(),
            StandardOfLiving { current_level: 4.0, quality_history: vec![] },
            Morale { current: 100.0, base: 50.0 },
        )).id();

        // Consume a low-quality item (e.g., nutrient paste instead of glitter-steak)
        app.world_mut().send_event(ConsumeItemEvent {
            consumer: pop_id,
            item_quality: 1.0,
        });
        app.update();

        // Pop should suffer a mood penalty for the downgrade
        let morale = app.world().get::<Morale>(pop_id).unwrap();
        assert!(morale.current < 100.0);
    }

    #[test]
    fn test_standard_slowly_decays_without_high_quality() {
        let mut app = App::new();
        app.insert_resource(SimulationTime { ticks: 0 });

        let pop_id = app.world_mut().spawn((
            Pop::default(),
            StandardOfLiving { current_level: 5.0, quality_history: vec![] },
        )).id();

        // Advance time significantly without high-quality consumption
        app.world_mut().resource_mut::<SimulationTime>().ticks = 50_000;
        app.update();

        // Standard of living should slowly return to a baseline over time
        let sol = app.world().get::<StandardOfLiving>(pop_id).unwrap();
        assert!(sol.current_level < 5.0);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Pop;

#[derive(Component, Default)]
pub struct Morale {
    pub current: f32,
    pub base: f32,
}

#[derive(Component)]
pub struct StandardOfLiving {
    pub current_level: f32,
    pub quality_history: Vec<f32>,
}

impl Default for StandardOfLiving {
    fn default() -> Self {
        Self {
            current_level: 1.0,
            quality_history: Vec::with_capacity(10),
        }
    }
}

#[derive(Event)]
pub struct ConsumeItemEvent {
    pub consumer: Entity,
    pub item_quality: f32,
}

pub fn process_consumption_quality(
    mut events: EventReader<ConsumeItemEvent>,
    mut query: Query<(&mut StandardOfLiving, &mut Morale)>,
) {
    for event in events.read() {
        if let Ok((mut sol, mut morale)) = query.get_mut(event.consumer) {
            // Track history
            sol.quality_history.push(event.item_quality);
            if sol.quality_history.len() > 10 {
                sol.quality_history.remove(0);
            }

            // Calculate new standard based on moving average
            let sum: f32 = sol.quality_history.iter().sum();
            let avg = sum / sol.quality_history.len() as f32;

            // Slowly drag current level toward average
            sol.current_level = (sol.current_level * 0.9) + (avg * 0.1);

            // If item is significantly below current standard, apply massive penalty
            if event.item_quality < sol.current_level - 1.0 {
                let penalty = (sol.current_level - event.item_quality) * 10.0;
                morale.current -= penalty;
            } else if event.item_quality > sol.current_level {
                // Small boost for exceeding standard
                morale.current += 5.0;
            }
        }
    }
}

pub fn decay_standard_of_living(
    mut query: Query<&mut StandardOfLiving>,
    time: Res<SimulationTime>,
) {
    if time.ticks % 1000 == 0 {
        for mut sol in query.iter_mut() {
            // Slowly decay towards 1.0 (baseline) over a long period
            if sol.current_level > 1.0 {
                sol.current_level -= 0.05;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Integration**: Tie the `ConsumeItemEvent` into existing actions like `EatAction` or `SleepAction` so that food and bed quality naturally trigger this check.
- **Traits**: Link the rate of standard decay/increase to Pop `Traits`. A "Frugal" pop might never raise their standard, while a "Decadent" pop raises theirs instantly.
- **Chronicle**: If a large portion of the colony suffers a Hedonic penalty simultaneously (e.g., a "Nutrient Paste Riot"), emit a `ChronicleEvent` for the history log.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Consuming high-quality items over time permanently raises `current_level`.
- [ ] Consuming low-quality items after raising the standard immediately reduces `Morale`.
- [ ] `StandardOfLiving` slowly decays toward 1.0 if not constantly reinforced.

## Technical Guidance

- Use a moving average or weighted average for `quality_history` to prevent a single high-quality meal from immediately resetting a pop's expectations.
- Make sure `item_quality` scales consistently across different item types (e.g., a "Basic Meal" is 1.0, "Luxury Bed" is 5.0).

## Questions

*Builder: add questions here if spec is unclear.*
