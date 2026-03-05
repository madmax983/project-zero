# Specification: The Sentient Standard (Layer 3 -> 1)

## 1. Overview
The Sentient Standard is a metric established by the Galactic Council (Layer 3) that dictates the expected quality of life for any civilized colony. If a colony's metrics (such as Luxury, Education, or Entertainment) fall below this standard, Pops gain a stacking "Left Behind" stress debuff, demanding the colony catch up to the galactic standard. This introduces a tension between maintaining a hyper-efficient, specialized economy and appeasing the cultural expectations of the wider galaxy.

## 2. Dependencies
- `003` Population Basics (Pop needs, StressTracker).
- `064` Room Quality (proxy for Luxury/Standard of Living).
- `010` Chronicle System (for receiving the "Glitterworld Broadcast" event that activates the standard).

## 3. RED Phase: Tests First

```rust
// tests/sentient_standard_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::pop::{Pop, StressTracker};
    use scale::layer1::social::SentientStandard;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems and resources
        world.insert_resource(SentientStandard {
            expected_luxury: 50.0,
            active: true,
        });
        world.insert_resource(ColonyMetrics {
            current_luxury: 30.0,
        });
        world
    }

    #[test]
    fn test_pop_gains_stress_when_below_standard() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            StressTracker::new(),
        )).id();

        // Run the system that evaluates the Sentient Standard
        world.run_system_once(apply_sentient_standard_stress_system);

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Luxury is 20 below standard, should generate proportional stress
        assert!(stress.accumulated_stress > f32::EPSILON);
    }

    #[test]
    fn test_pop_does_not_gain_stress_when_above_standard() {
        let mut world = setup_world();

        // Exceed the standard
        world.resource_mut::<ColonyMetrics>().current_luxury = 60.0;

        let pop = world.spawn((
            Pop,
            StressTracker::new(),
        )).id();

        world.run_system_once(apply_sentient_standard_stress_system);

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Should not gain stress from Sentient Standard
        assert!((stress.accumulated_stress - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sentient_standard_inactive_causes_no_stress() {
        let mut world = setup_world();

        // Deactivate the standard (e.g., broadcast hasn't reached colony yet)
        world.resource_mut::<SentientStandard>().active = false;

        let pop = world.spawn((
            Pop,
            StressTracker::new(),
        )).id();

        world.run_system_once(apply_sentient_standard_stress_system);

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Inactive standard shouldn't cause stress despite metrics being low
        assert!((stress.accumulated_stress - 0.0).abs() < f32::EPSILON);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/social/sentient_standard.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, StressTracker};

#[derive(Resource, Debug, Clone, Copy)]
pub struct SentientStandard {
    pub expected_luxury: f32,
    pub active: bool,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ColonyMetrics {
    pub current_luxury: f32,
}

pub fn apply_sentient_standard_stress_system(
    standard: Option<Res<SentientStandard>>,
    metrics: Option<Res<ColonyMetrics>>,
    mut pop_query: Query<&mut StressTracker, With<Pop>>,
) {
    if let (Some(standard), Some(metrics)) = (standard, metrics) {
        if !standard.active {
            return;
        }

        let deficit = standard.expected_luxury - metrics.current_luxury;

        if deficit > f32::EPSILON {
            // Apply a minor constant stress tick based on the deficit
            let stress_penalty = deficit * 0.01;

            for mut tracker in pop_query.iter_mut() {
                tracker.accumulated_stress += stress_penalty;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Component Marker:** Instead of blindly applying stress to all Pops, introduce a `LeftBehind` marker component or trait that gets attached to Pops. This allows specific Pops (e.g., "Survivalist" traits) to ignore the standard, or allows the UI to easily query who is affected.
- **Metric Calculation:** `ColonyMetrics` needs to be populated by another system that calculates average room quality, leisure building availability, etc. For now, it's a simple Resource.
- **Balancing:** The stress penalty (`deficit * 0.01`) should be moved to a configuration constant (e.g., in `crate::layer1::balance`) rather than hardcoded.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/social/sentient_standard.rs`.
- [ ] Pops gain stress when `ColonyMetrics` is below the active `SentientStandard`.
- [ ] Pops do not gain stress when the standard is inactive or met.

## 7. Technical Guidance
- **System Registration:** Register `apply_sentient_standard_stress_system` in the appropriate schedule (e.g., `Update` or a periodic tick schedule) after `ColonyMetrics` are updated.
- **Resource Initialization:** Ensure `SentientStandard` and `ColonyMetrics` are properly initialized. The `active` flag should start as `false` and only become `true` when triggered by a Layer 3 event or Chronicle milestone.
- **Float Comparison:** Strictly use epsilon checks (`(a - b).abs() < f32::EPSILON`) for float comparisons in tests.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
