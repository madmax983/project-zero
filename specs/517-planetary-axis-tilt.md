# Spec 517: Planetary Axis Tilt

## 1. Overview
Not all worlds are Earth-like. The seasons can be murder. Planets generate with an "Axial Tilt" value (0-90 degrees). Low tilt = no seasons (eternal spring/autumn). High tilt = extreme seasons (unlivable heat in summer, deep freeze in winter). This creates a tension between settling the "Goldilocks" zone (boring resources) or the Extreme zones (rare resources, harsh cycles).

**Layer:** 2 -> 1
**Fantasy:** The seasons can be murder.

## 2. Dependencies
- `095` System Generation (Planetary traits)
- `079` Weather Events (Seasonal integration)
- `140` Thermal Management (Temperature application)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_low_tilt_results_in_mild_temperature_swing() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(PlanetaryTilt { degrees: 5.0 });
        app.insert_resource(Season { current: SeasonType::Summer });
        app.add_systems(Update, calculate_seasonal_temperature_swing);

        let mut temp = TemperatureModifier::default();

        // Act
        app.world_mut().resource_mut::<Season>().current = SeasonType::Summer;
        app.update();
        let summer_temp = app.world().resource::<TemperatureModifier>().offset;

        app.world_mut().resource_mut::<Season>().current = SeasonType::Winter;
        app.update();
        let winter_temp = app.world().resource::<TemperatureModifier>().offset;

        // Assert
        assert!((summer_temp - winter_temp).abs() < 10.0, "Low tilt should have small temp delta");
    }

    #[test]
    fn test_high_tilt_results_in_extreme_temperature_swing() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(PlanetaryTilt { degrees: 80.0 });
        app.insert_resource(Season { current: SeasonType::Summer });
        app.insert_resource(TemperatureModifier::default());
        app.add_systems(Update, calculate_seasonal_temperature_swing);

        // Act
        app.world_mut().resource_mut::<Season>().current = SeasonType::Summer;
        app.update();
        let summer_temp = app.world().resource::<TemperatureModifier>().offset;

        app.world_mut().resource_mut::<Season>().current = SeasonType::Winter;
        app.update();
        let winter_temp = app.world().resource::<TemperatureModifier>().offset;

        // Assert
        assert!((summer_temp - winter_temp).abs() > 50.0, "High tilt should have large temp delta");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct PlanetaryTilt {
    pub degrees: f32, // 0.0 to 90.0
}

#[derive(Resource)]
pub struct Season {
    pub current: SeasonType,
}

#[derive(PartialEq, Clone, Copy)]
pub enum SeasonType {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Resource, Default)]
pub struct TemperatureModifier {
    pub offset: f32,
}

pub fn calculate_seasonal_temperature_swing(
    tilt: Res<PlanetaryTilt>,
    season: Res<Season>,
    mut temp_mod: ResMut<TemperatureModifier>,
) {
    // Basic calculation:
    // Base temp swing multiplier based on tilt (e.g., 1 degree tilt = 0.5 degrees swing)
    let max_swing = tilt.degrees * 0.8;

    match season.current {
        SeasonType::Summer => temp_mod.offset = max_swing,
        SeasonType::Winter => temp_mod.offset = -max_swing,
        SeasonType::Spring | SeasonType::Autumn => temp_mod.offset = 0.0,
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - The temperature swing logic should probably integrate directly with `Layer1Climate` or global weather settings, rather than just floating as a solitary `TemperatureModifier`.
  - The calculation should ideally use a sine wave representing the planet's orbit around the star, factoring in orbital eccentricity along with axial tilt.
- **Code Smells:**
  - Hardcoded `0.8` multiplier. This should be exposed as a configurable constant (`TILT_TO_TEMP_SCALAR`).
- **Performance:**
  - Updating temperature once per season change is highly performant. If it changes daily (gradual curve), ensure the math isn't overly complex.
- **API Improvements:**
  - Use `Layer2Planet` component instead of `PlanetaryTilt` resource to store the tilt data if supporting multi-planet gameplay.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for planetary tilt mechanics.
- [ ] Planets generated with `PlanetaryTilt` resource.
- [ ] High tilt produces massive temperature differentials between Summer and Winter.
- [ ] Low tilt produces near-constant temperatures year-round.

## 7. Technical Guidance
- **Gotchas:** Ensure the base temperature of the planet is applied *before* the seasonal modifier. A highly tilted ice planet will have a slightly less cold summer and an absolutely unlivable winter.
- **Integration Points:** Connect `PlanetaryTilt` to system generation (`src/layer2/system_gen.rs`) and pass the calculated temperature offset down to `src/layer1/thermal.rs`.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
