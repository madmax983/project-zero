# 920 - The Parasitic Artifact

## 1. Overview

**Layer:** 1
**Fantasy:** Digging up a treasure that slowly eats your colony's culture.
**Mechanic:** A mined xeno-artifact provides a passive boost to nearby building efficiency, but slowly 're-writes' the `UtilityWeights` of pops who spend time near it, increasing their desire to admire the artifact over essential needs.

## 2. Dependencies

- `UtilityWeights` (Layer 1 Needs)
- Building efficiency systems

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components for testing
    #[derive(Component)]
    struct Building { efficiency: f32 }

    #[derive(Component, Default)]
    struct UtilityWeights {
        eat: f32,
        sleep: f32,
        obsession: f32,
    }

    #[test]
    fn test_parasitic_artifact_boosts_efficiency() {
        let mut app = App::new();
        app.add_systems(Update, apply_parasitic_artifact_efficiency_system);

        let artifact = app.world_mut().spawn((
            ParasiticArtifact { range: 5.0, efficiency_boost: 1.5, obsession_rate: 0.1 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let building = app.world_mut().spawn((
            Building { efficiency: 1.0 },
            Transform::from_xyz(2.0, 0.0, 0.0),
        )).id();

        app.update();

        assert_eq!(app.world().get::<Building>(building).unwrap().efficiency, 1.5);
    }

    #[test]
    fn test_parasitic_artifact_rewrites_utility_weights() {
        let mut app = App::new();
        app.add_systems(Update, parasitic_artifact_obsession_system);
        app.insert_resource(Time::default() as Time);

        let artifact = app.world_mut().spawn((
            ParasiticArtifact { range: 5.0, efficiency_boost: 1.5, obsession_rate: 0.1 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            UtilityWeights::default(),
            Transform::from_xyz(2.0, 0.0, 0.0),
        )).id();

        // Advance time
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(10));

        app.update();

        let weights = app.world().get::<UtilityWeights>(pop).unwrap();
        assert!(weights.obsession > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ParasiticArtifact {
    pub range: f32,
    pub efficiency_boost: f32,
    pub obsession_rate: f32,
}

// Minimal implementation assuming Building and UtilityWeights exist in scope
// pub fn apply_parasitic_artifact_efficiency_system(...) { ... }
// pub fn parasitic_artifact_obsession_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: Spatial distance check $O(N \times M)$ should be optimized.
- **Code Smells**: Hardcoded base efficiency values might need a proper multiplicative modifier system.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code

## 7. Technical Guidance

- Integrate `ParasiticArtifact` with the existing `UtilityWeights` logic.
- Register systems appropriately in `simulation.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
