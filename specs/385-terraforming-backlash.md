# 385: Terraforming Backlash

## 1. Overview
**Layer:** 2
**Fantasy:** The planet has an immune system, and you are the virus.
**Mechanic:** Aggressive terraforming (atmosphere injection, rapid heating) triggers "Planetary Defense" events: Super-storms, seismic destabilization, or awakening dormant mega-fauna.
**Emergence:** You try to melt the ice caps to get water, and the release of ancient bacteria kills half your population.
**Tension:** Fast, violent adaptation (Terraforming) vs. Slow, biological adaptation (Genetics).

## 2. Dependencies
- Layer 2 Terraforming values (`Temperature`, `Atmosphere`).
- Event dispatch system for hazards or plagues.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_rapid_temperature_change_triggers_backlash() {
        // Arrange
        let mut app = App::new();
        app.add_event::<BacklashEvent>();
        app.insert_resource(TerraformingState {
            temperature: 20.0,
            last_temperature: 10.0, // 10 degree jump in one check cycle
        });
        app.add_systems(Update, check_terraforming_backlash_system);

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<BacklashEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.len(events), 1, "Backlash event should be triggered for rapid heating");
    }

    #[test]
    fn test_slow_temperature_change_is_safe() {
        // Arrange
        let mut app = App::new();
        app.add_event::<BacklashEvent>();
        app.insert_resource(TerraformingState {
            temperature: 11.0,
            last_temperature: 10.0, // 1 degree jump
        });
        app.add_systems(Update, check_terraforming_backlash_system);

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<BacklashEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.len(events), 0, "No backlash for slow changes");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct TerraformingState {
    pub temperature: f32,
    pub last_temperature: f32,
}

#[derive(Event)]
pub struct BacklashEvent {
    pub severity: f32,
}

pub fn check_terraforming_backlash_system(
    mut state: ResMut<TerraformingState>,
    mut events: EventWriter<BacklashEvent>,
) {
    let temp_delta = (state.temperature - state.last_temperature).abs();

    // Threshold for rapid change (e.g., > 5 degrees per check)
    if temp_delta > 5.0 {
        events.send(BacklashEvent { severity: temp_delta });
    }

    // Update last_temperature to prevent repeating
    state.last_temperature = state.temperature;
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Incorporate multiple vectors (pressure, toxicity) into a generalized `TerraformingStress` value.
- **Code Smells:** `last_temperature` tracking is brittle. Instead, track `TerraformingVelocity` over a time window.
- **Performance:** Minimal overhead. Can be checked on a slow interval rather than every frame.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Rapid terraforming triggers a backlash event.
- [ ] Slow terraforming does not trigger the event.

## 7. Technical Guidance
- Integrate into the hazard generation system. A `BacklashEvent` should spawn a super-storm or an earthquake depending on its type and severity.
- Ensure the backlash check aligns with whatever system modifies `TerraformingState`.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
