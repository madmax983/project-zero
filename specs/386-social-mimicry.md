# 386 - Social Mimicry

## 1. Overview
**Layer:** 1
**Fantasy:** Pops are sheep. Trends spread like viruses.
**Mechanic:** If a "High Status" pop (high job tier/social skill) adopts a behavior (e.g., eating a specific luxury food, wearing a specific hat), nearby pops have a chance to copy it, changing their preferences.
**Emergence:** The Governor develops a taste for "Rat-on-a-stick". Suddenly, it becomes the fashionable high-cuisine, causing a luxury food market crash and a rat extinction event.
**Tension:** Do you indulge the elite's expensive tastes knowing the masses will demand the same?

## 2. Dependencies
- `Pop` component
- `SocialStatus` component (or similar prestige value)
- `Needs` or `Preferences` tracking (what they eat/wear)
- Bevy's spatial query or interaction events

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, SocialStatus, Preferences, ItemType};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, social_mimicry_system);
        app
    }

    #[test]
    fn test_low_status_mimics_high_status() {
        let mut app = setup_app();

        // High status pop eating LuxuryFood
        let high_status_pop = app.world_mut().spawn((
            Pop,
            SocialStatus { level: 10 },
            Preferences { favorite_food: Some(ItemType::LuxuryFood) },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        // Low status pop eating BasicFood
        let low_status_pop = app.world_mut().spawn((
            Pop,
            SocialStatus { level: 1 },
            Preferences { favorite_food: Some(ItemType::BasicFood) },
            Transform::from_xyz(1.0, 0.0, 0.0), // Adjacent
        )).id();

        app.update();

        // The low status pop should mimic the high status pop's preference
        let low_status_prefs = app.world().get::<Preferences>(low_status_pop).unwrap();
        assert_eq!(low_status_prefs.favorite_food, Some(ItemType::LuxuryFood));
    }

    #[test]
    fn test_high_status_does_not_mimic_low_status() {
        let mut app = setup_app();

        let low_status_pop = app.world_mut().spawn((
            Pop,
            SocialStatus { level: 1 },
            Preferences { favorite_food: Some(ItemType::BasicFood) },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let high_status_pop = app.world_mut().spawn((
            Pop,
            SocialStatus { level: 10 },
            Preferences { favorite_food: Some(ItemType::LuxuryFood) },
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        app.update();

        // High status pop retains their own preference
        let high_status_prefs = app.world().get::<Preferences>(high_status_pop).unwrap();
        assert_eq!(high_status_prefs.favorite_food, Some(ItemType::LuxuryFood));
    }

    #[test]
    fn test_mimicry_out_of_range() {
        let mut app = setup_app();

        let high_status_pop = app.world_mut().spawn((
            Pop,
            SocialStatus { level: 10 },
            Preferences { favorite_food: Some(ItemType::LuxuryFood) },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let low_status_pop = app.world_mut().spawn((
            Pop,
            SocialStatus { level: 1 },
            Preferences { favorite_food: Some(ItemType::BasicFood) },
            Transform::from_xyz(100.0, 0.0, 0.0), // Too far away
        )).id();

        app.update();

        // Low status pop does not mimic due to distance
        let low_status_prefs = app.world().get::<Preferences>(low_status_pop).unwrap();
        assert_eq!(low_status_prefs.favorite_food, Some(ItemType::BasicFood));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, SocialStatus, Preferences, ItemType};

const MIMICRY_RADIUS: f32 = 5.0;

pub fn social_mimicry_system(
    mut query: Query<(Entity, &mut Preferences, &SocialStatus, &Transform), With<Pop>>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(e1, mut p1, s1, t1), (e2, mut p2, s2, t2)]) = combinations.fetch_next() {
        if t1.translation.distance(t2.translation) <= MIMICRY_RADIUS {
            if s1.level > s2.level {
                p2.favorite_food = p1.favorite_food.clone();
            } else if s2.level > s1.level {
                p1.favorite_food = p2.favorite_food.clone();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently uses O(N^2) `iter_combinations_mut` check. This is slow for many pops. A spatial grid hash or a quadtree should be used to restrict interaction checks to nearby entities.
- Introduce a cooldown or probability component so mimicry isn't instantaneous or guaranteed every tick.
- Add event emission when a preference changes for UI notifications.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Social Mimicry successfully propagates preferences from high-status to low-status pops when nearby.

## 7. Technical Guidance
- Integrate into the `Layer1SystemSet::Observation` or equivalent set.
- A `SocialStatus` component needs to be well-defined if it doesn't already exist.

## 8. Questions
- Should mimicry apply to clothing/equipment as well, or just food preferences?
