# 910: Dream Shaping

## 1. Overview
Colonies can construct "Somnambulator" towers to project behavioral themes (e.g., Compliance, Valor, Consumption) into the minds of sleeping Pops. While this accelerates ethic shifts and experience gain, it drastically reduces sleep quality and introduces the risk of population-wide nightmare events, creating severe waking behavioral disturbances.

## 2. Dependencies
- `layer1::needs::SleepQuality`
- `layer1::mind::Ethics`
- `layer1::infrastructure::SomnambulatorTower`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::SleepQuality;

    #[test]
    fn test_somnambulator_tower_reduces_sleep_quality() {
        let mut app = App::new();
        app.add_systems(Update, apply_dream_shaping);

        let tower = app.world_mut().spawn(SomnambulatorTower {
            active_theme: Some(DreamTheme::Valor),
            range: 10.0,
        }).id();

        let pop = app.world_mut().spawn((
            SleepQuality { value: 1.0 },
            Position { x: 0.0, y: 0.0 },
            Sleeping, // A marker component
        )).id();

        // Assume tower is at 0,0 (in range)
        app.world_mut().entity_mut(tower).insert(Position { x: 0.0, y: 0.0 });

        // Act
        app.update();

        // Assert
        let sleep = app.world().get::<SleepQuality>(pop).unwrap();
        assert!(sleep.value < 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Simplest implementation to turn tests green
```

## 5. REFACTOR Phase: Quality & Design
- Implement a spatial query using `bevy_spatial` or distance checks to ensure only sleeping pops within tower range are affected.
- Cache active towers to avoid O(N*M) checks between pops and towers.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Active somnambulator towers decrease `SleepQuality` of nearby sleeping pops

## 7. Technical Guidance
- Integrate with `layer1::bureaucracy_of_sleep` if present, ensuring Dream Shaping plays nicely with fatigue trackers.
- A "Nightmare" side effect could spawn an immediate stress/panic action for affected pops.

## 8. Questions
*Builder: add questions here if spec is unclear.*
