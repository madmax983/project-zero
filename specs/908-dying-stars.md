# 908: Dying Stars

## 1. Overview
The local star is unstable, adding a ticking clock and forcing a race to escape. It progresses through stages ("Flare Up", "Expansion", "Nova"). Each stage significantly changes planetary conditions by escalating heat and radiation, fundamentally altering the "Safe Core Worlds" into uninhabitable hellscapes.

## 2. Dependencies
- `layer2::system_map::StarNode`
- `layer2::environment::PlanetaryConditions`
- `layer1::environment::HeatGrid`
- `layer1::environment::RadiationGrid`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::environment::PlanetaryConditions;

    #[test]
    fn test_star_stage_progression_increases_heat_and_rads() {
        let mut app = App::new();
        app.add_systems(Update, process_dying_star_stage);

        let star = app.world_mut().spawn(StarNode {
            stage: StarStage::Stable,
            time_to_next_stage: 10.0,
        }).id();

        let planet = app.world_mut().spawn((
            PlanetaryConditions { base_heat: 10.0, base_radiation: 0.0 },
            Orbiting(star),
        )).id();

        // Act: Advance to Flare Up
        app.world_mut().get_mut::<StarNode>(star).unwrap().stage = StarStage::FlareUp;
        app.update();

        // Assert
        let conditions = app.world().get::<PlanetaryConditions>(planet).unwrap();
        assert!(conditions.base_heat > 10.0);
        assert!(conditions.base_radiation > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Simplest implementation to turn tests green
```

## 5. REFACTOR Phase: Quality & Design
- Create an observer system that only triggers planetary condition updates when the star's stage changes.
- Consider moving the `Orbiting` traversal to a broader orbital mechanics update to save queries.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Planetary conditions respond correctly to star stages

## 7. Technical Guidance
- Ensure that updating planetary base values triggers recalculation of `HeatGrid` and `RadiationGrid` on Layer 1 if a colony is active on that planet.
- Consider utilizing a separate event `StarStageChangedEvent` to keep systems decoupled.

## 8. Questions
*Builder: add questions here if spec is unclear.*
