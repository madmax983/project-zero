# Specification: Psychic Background Radiation

## 1. Overview
**Psychic Background Radiation** is a Cross-Layer mechanic where the ambient "psychic noise" of a Layer 2 system sector affects the Pops on Layer 1. High psychic background noise reduces sleep efficiency and increases the rate of stress accumulation, simulating an environment where "space is screaming." This introduces external, unavoidable pressures on the colony that the player must mitigate through architecture or social policies.

## 2. Dependencies
- `003` Population Basics (Sleep need, StressTracker).
- `152` Orbital Stations (System map traversal).
- `063` Atmospheric Simulation (if we tie it to weather, though it acts like an atmospheric condition).

## 3. RED Phase: Tests First

```rust
// tests/psychic_radiation_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::pop::{Pop, StressTracker, SleepNeed};
    use scale::layer1::psychic::PsychicBackground;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(PsychicBackground { level: 0.0 });
        world
    }

    #[test]
    fn test_high_psychic_background_increases_stress() {
        let mut world = setup_world();
        world.resource_mut::<PsychicBackground>().level = 100.0;

        let pop = world.spawn((
            Pop,
            StressTracker::new(),
        )).id();

        // Run system
        world.run_system_once(apply_psychic_background_system);

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress > f32::EPSILON, "Pops should gain stress from high psychic radiation");
    }

    #[test]
    fn test_high_psychic_background_reduces_sleep_efficiency() {
        let mut world = setup_world();
        world.resource_mut::<PsychicBackground>().level = 100.0;

        let mut sleep = SleepNeed::new();
        sleep.value = 50.0; // Half asleep
        let pop = world.spawn((
            Pop,
            sleep,
        )).id();

        // Let's assume there's a system that applies the sleep efficiency modifier
        world.run_system_once(apply_psychic_background_system);

        let sleep = world.get::<SleepNeed>(pop).unwrap();
        // The modifier should be less than 1.0 (e.g., 0.5) making sleep less effective
        assert!(sleep.efficiency_modifier < 1.0, "Sleep efficiency should be reduced by psychic radiation");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/psychic.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, StressTracker, SleepNeed};

#[derive(Resource, Default)]
pub struct PsychicBackground {
    pub level: f32, // 0.0 to 100.0
}

pub fn apply_psychic_background_system(
    bg: Option<Res<PsychicBackground>>,
    mut query: Query<(&mut StressTracker, &mut SleepNeed), With<Pop>>,
) {
    if let Some(bg) = bg {
        if bg.level <= 0.0 {
            return;
        }

        let stress_penalty = bg.level * 0.001; // Scale factor
        let sleep_penalty = 1.0 - (bg.level * 0.005).clamp(0.0, 0.9); // Up to 90% reduction

        for (mut stress, mut sleep) in query.iter_mut() {
            stress.accumulated_stress += stress_penalty;
            sleep.efficiency_modifier = sleep_penalty;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Integration:** The `PsychicBackground` level should be updated by a Layer 2 system that checks the system's position within the galaxy (e.g., moving through a nebula).
- **Mitigation:** Add specific building materials or traits (e.g., "Psychically Null" traits or "Lead-lined Walls") that negate or reduce this penalty for Pops inside.
- **UI:** The background level should be visible to the player as an environmental hazard overlay.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/psychic.rs`.
- [ ] High psychic background levels increase stress accumulation on all Pops.
- [ ] High psychic background levels reduce sleep efficiency on all Pops.

## 7. Technical Guidance
- **System Registration:** Add `apply_psychic_background_system` to the main update loop, likely in the same stage where environmental needs are calculated.
- **Component Modifications:** You may need to add `efficiency_modifier` to `SleepNeed` if it doesn't already exist, and incorporate it into the sleep restoration logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
