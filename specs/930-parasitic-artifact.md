# 930 - The Parasitic Artifact

## 1. Overview

**Layer:** 1
**Fantasy:** Digging up a treasure that slowly eats your colony's culture.
**Mechanic:** A mined xeno-artifact provides a massive passive boost to nearby building efficiency, but slowly 're-writes' the UtilityWeights (personality) of pops who spend time near it, making them obsessed with staring at the artifact instead of sleeping or socializing.
**Emergence:** You place the artifact in your main industrial zone to boost production. Months later, the entire factory staff starves to death because their 'need to admire the artifact' score overrode their 'need to eat' score.
**Tension:** Exploiting the massive efficiency boost versus the slow, insidious destruction of your workforce's sanity.

## 2. Dependencies

- Utility AI / UtilityWeights system
- Mining / Artifact discovery
- Building efficiency modifiers
- Spatial queries (distance from artifact)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct ParasiticArtifact { influence_radius: f32, efficiency_boost: f32 }

    #[derive(Component)]
    struct Transform { translation: Vec3 }

    #[derive(Component)]
    struct Building { efficiency: f32 }

    #[derive(Component)]
    struct PopUtilityWeights { admire_artifact: f32, eat: f32 }

    #[test]
    fn test_artifact_boosts_building_efficiency() {
        let mut app = App::new();
        app.add_systems(Update, apply_artifact_efficiency_system);

        app.world_mut().spawn((
            ParasiticArtifact { influence_radius: 10.0, efficiency_boost: 2.0 },
            Transform { translation: Vec3::new(0.0, 0.0, 0.0) },
        ));

        let building = app.world_mut().spawn((
            Building { efficiency: 1.0 },
            Transform { translation: Vec3::new(5.0, 0.0, 0.0) },
        )).id();

        app.update();

        // Building should have increased efficiency due to proximity
        let updated_building = app.world().get::<Building>(building).unwrap();
        assert_eq!(updated_building.efficiency, 2.0);
    }

    #[test]
    fn test_artifact_rewrites_pop_utility_weights() {
        let mut app = App::new();
        app.add_systems(Update, apply_artifact_psychological_effect_system);

        app.world_mut().spawn((
            ParasiticArtifact { influence_radius: 10.0, efficiency_boost: 2.0 },
            Transform { translation: Vec3::new(0.0, 0.0, 0.0) },
        ));

        let pop = app.world_mut().spawn((
            PopUtilityWeights { admire_artifact: 0.1, eat: 0.9 },
            Transform { translation: Vec3::new(2.0, 0.0, 0.0) },
        )).id();

        app.update();

        // Pop's admire weight should increase, eating weight should decrease
        let updated_weights = app.world().get::<PopUtilityWeights>(pop).unwrap();
        assert!(updated_weights.admire_artifact > 0.1);
        assert!(updated_weights.eat < 0.9);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn apply_artifact_efficiency_system(...) { ... }
// pub fn apply_artifact_psychological_effect_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Ensure the efficiency multiplier logic is non-destructive (e.g. recalculating it each frame instead of modifying the base value directly) to avoid infinite scaling.
- Use `Time::delta_seconds()` for the psychological rewriting to make it a gradual, creeping change over time rather than instant.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Artifacts boost nearby building efficiency.
- [ ] Artifacts gradually alter nearby Pops' utility weights to prioritize admiring it.

## 7. Technical Guidance

- You will need a spatial query or distance check between the `ParasiticArtifact` and `Pop`/`Building` entities.
- Add a new action type `ActionType::AdmireArtifact` to the Utility AI to make pops actually go stare at it.

## 8. Questions
*Builder: add questions here if spec is unclear.*
