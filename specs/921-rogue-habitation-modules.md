# 921 - Rogue Habitation Modules

## 1. Overview

**Layer:** 2
**Fantasy:** A neighborhood simply deciding to leave the planet.
**Mechanic:** High-tier orbital housing units have emergency thrusters. If the inhabitants' unrest reaches critical mass, they fire the thrusters, decouple from the main station, and become an independent, mobile mini-station.

## 2. Dependencies

- Layer 2 Station/Housing components
- Unrest tracking system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct HabitationModule { is_decoupled: bool }

    #[derive(Component)]
    struct UnrestTracker { level: f32, critical_threshold: f32 }

    #[derive(Component)]
    struct MobileStation;

    #[test]
    fn test_high_unrest_decouples_module() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_habitation_unrest_system);

        let module = app.world_mut().spawn((
            HabitationModule { is_decoupled: false },
            UnrestTracker { level: 95.0, critical_threshold: 90.0 },
        )).id();

        app.update();

        let hab = app.world().get::<HabitationModule>(module).unwrap();
        assert!(hab.is_decoupled);
        assert!(app.world().get::<MobileStation>(module).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components omitted for brevity
// pub fn evaluate_habitation_unrest_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Ensure decoupling doesn't orphan child entities (e.g., Pops living inside).
- Use events (`ModuleDecoupledEvent`) to trigger broader narrative or economic consequences.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] Test coverage ≥85%
- [ ] Module transforms into a mobile station upon critical unrest.

## 7. Technical Guidance

- Use `Commands` to insert `MobileStation` and remove static station constraints.

## 8. Questions
*Builder: add questions here if spec is unclear.*
