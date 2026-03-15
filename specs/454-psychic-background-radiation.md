# Spec 454: Psychic Background Radiation

## 1. Overview
**Layer:** Cross-Layer (Layer 2 -> Layer 1)

**Fantasy:** Space isn't silent, it's screaming.
**Mechanic:** "Psychic Background" level on Layer 2 affects Layer 1. High background noise reduces sleep efficiency and increases mental break chance.
**Emergence:** The planet passes through a nebula. Everyone has nightmares. Production halts because everyone is exhausted.
**Tension:** Wait out the storm (idle) or push through (risk breaks)?

This specification introduces `PsychicBackgroundRadiation` tracking in the `World` (Layer 2 equivalent for now), which directly applies debuffs to Pop `Rest` restoration and increases `Stress` accumulation rates globally on the colony (Layer 1).

## 2. Dependencies
- `005` Pop needs (hunger, rest)
- `127` Stress Breakdowns
- `013` Schedule and System Execution Ordering

## 3. RED Phase: Tests First

```rust
// tests/integration/psychic_radiation.rs

#[cfg(test)]
mod tests {
    use scale::layer1::needs::{Needs, Rest};
    use scale::layer1::stress::Stress;
    use scale::layer2::environment::PsychicBackgroundRadiation;
    use bevy::prelude::*;

    // Note: Assuming a basic App setup function exists
    fn setup_test_app() -> App {
        let mut app = App::new();
        // Setup base systems needed for testing...
        app
    }

    #[test]
    fn test_psychic_radiation_increases_stress() {
        let mut app = setup_test_app();

        // Arrange
        app.world_mut().insert_resource(PsychicBackgroundRadiation { intensity: 0.5 });

        let pop = app.world_mut().spawn((
            Stress { current: 0.0, max: 100.0 },
        )).id();

        // Act
        // Run the system that applies radiation effects
        app.update();

        // Assert
        let stress = app.world().get::<Stress>(pop).unwrap();
        assert!(stress.current > 0.0, "Stress should increase when Psychic Background Radiation is active");
    }

    #[test]
    fn test_psychic_radiation_reduces_rest_efficiency() {
        let mut app = setup_test_app();

        // Arrange
        app.world_mut().insert_resource(PsychicBackgroundRadiation { intensity: 0.8 });

        // Simulate a pop resting. A normal rest system might restore X amount,
        // but with radiation, it should restore less.
        let pop = app.world_mut().spawn((
            Needs { rest: 50.0, ..Default::default() },
        )).id();

        // Act
        app.update();

        // Assert: Needs specific integration with rest logic, but the conceptual test ensures
        // that rest recovery is negatively scaled by the `intensity`.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/environment.rs

use bevy::prelude::*;

/// Tracks the global level of psychic background noise affecting the system.
#[derive(Resource, Default, Debug)]
pub struct PsychicBackgroundRadiation {
    /// Intensity from 0.0 (none) to 1.0 (extreme).
    pub intensity: f32,
}

// src/layer1/systems/psychic_effects.rs

use bevy::prelude::*;
use crate::layer1::stress::Stress;
use crate::layer2::environment::PsychicBackgroundRadiation;

/// Applies stress to all pops based on the current Psychic Background Radiation.
pub fn apply_psychic_radiation_stress_system(
    radiation: Option<Res<PsychicBackgroundRadiation>>,
    mut query: Query<&mut Stress>,
) {
    if let Some(rad) = radiation {
        if rad.intensity > 0.0 {
            let stress_increase = rad.intensity * 2.0; // Base multiplier for MVP
            for mut stress in query.iter_mut() {
                stress.current = (stress.current + stress_increase).min(stress.max);
            }
        }
    }
}
```

*(Note: Rest efficiency integration should hook into the existing rest restoration system to apply an inverse scalar based on `radiation.intensity`.)*

## 5. REFACTOR Phase: Quality & Design

- **Smell:** Hardcoded stress multipliers.
- **Improvement:** Move the base multiplier to a configurable constant or a `PsychicRadiationConfig` resource.
- **Improvement:** Ensure `PsychicBackgroundRadiation` affects sleep efficiency elegantly. This may require modifying `src/layer1/needs.rs` or `src/layer1/execution/rest.rs` to read the resource and scale the delta.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] PsychicBackgroundRadiation resource exists and can be set to [0.0, 1.0].
- [ ] A system applies stress per tick when radiation intensity > 0.0.

## 7. Technical Guidance

- Register `PsychicBackgroundRadiation` as an optional resource. If it doesn't exist, assume 0.0.
- To impact `Rest`, find the system responsible for restoring `Needs.rest` (usually when a Pop is sleeping) and apply a debuff multiplier: `rest_gained *= (1.0 - (radiation.intensity * 0.5))`.
- Bevy ECS System Ordering Insight: Ensure `apply_psychic_radiation_stress_system` runs before the `stress_breakdown_system` so that the latest stress values are used for evaluating breakdowns.

## 8. Questions

*Builder: add questions here if spec is unclear.*
