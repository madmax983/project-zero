# Spec 344: Seasonal Rhythms

## 1. Overview
The colony breathes with the planet through "Seasons." Seasons affect crop growth rates, temperature needs, and movement speed (snow/mud). This creates natural peaks and valleys in resource production, forcing the player to plan ahead and stockpile.

## 2. Dependencies
- Global time/calendar system (Layer 1/2)
- Farming/Crop growth system
- Pop movement speed/pathfinding
- Temperature mechanics (if implemented)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_season_affects_crop_growth_rate() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(SeasonTracker { current: Season::Winter });
        app.add_systems(Update, apply_seasonal_growth_modifiers);

        let crop = app.world_mut().spawn(CropGrowth {
            base_rate: 1.0,
            current_rate: 1.0,
        }).id();

        // Act
        app.update();

        // Assert
        let growth = app.world().get::<CropGrowth>(crop).unwrap();
        // Winter should significantly slow down or stop growth
        assert!(growth.current_rate < growth.base_rate);
    }

    #[test]
    fn test_season_transitions_over_time() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(SeasonTracker { current: Season::Autumn, days_remaining: 1 });
        app.add_systems(Update, advance_seasons_system);

        // Act
        app.update(); // 1 day passes

        // Assert
        let tracker = app.world().resource::<SeasonTracker>();
        assert_eq!(tracker.current, Season::Winter);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Resource)]
pub struct SeasonTracker {
    pub current: Season,
    pub days_remaining: u32,
}

#[derive(Component)]
pub struct CropGrowth {
    pub base_rate: f32,
    pub current_rate: f32,
}

pub fn apply_seasonal_growth_modifiers(
    season: Res<SeasonTracker>,
    mut query: Query<&mut CropGrowth>,
) {
    for mut crop in query.iter_mut() {
        crop.current_rate = match season.current {
            Season::Winter => crop.base_rate * 0.1,
            Season::Summer => crop.base_rate * 1.5,
            _ => crop.base_rate,
        };
    }
}

pub fn advance_seasons_system(
    mut season: ResMut<SeasonTracker>,
) {
    if season.days_remaining > 0 {
        season.days_remaining -= 1;
    } else {
        season.current = match season.current {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        };
        season.days_remaining = 30; // Assuming 30-day seasons for now
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Modifiers API**: Instead of hardcoding `0.1` or `1.5`, seasons should inject a multiplier into a central `ModifierTracker` on the crop entity.
- **Visuals**: Changing seasons should trigger visual changes (e.g., snow shader over tiles, different tree sprites).
- **Movement Speed**: Tie season-specific terrain modifiers (e.g., Mud in Spring, Snow in Winter) into pathfinding cost calculations.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for season tracking and modifier application.
- [ ] Seasons properly transition after a set duration.

## 7. Technical Guidance
- **Layer**: Layer 1 (Colony level).
- Consider making the length of seasons configurable per-planet (tying into "Planetary Quirks").
- A "Long Winter" could be an emergent narrative event. Provide an event hook `SeasonTransitionEvent` so other systems (like UI or chronicle) can react.

## 8. Questions
*Builder: add questions here if spec is unclear.*
