# 922 - Debt Gravity

## 1. Overview

**Layer:** Cross-layer (1 & 2)
**Fantasy:** Economic failure physically trapping you on a world.
**Mechanic:** High levels of collective planetary debt physically increase the mass/fuel cost required to launch ships from that colony.

## 2. Dependencies

- Economic Debt system
- Ship Launch/Logistics system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct PlanetaryDebt { amount: f32 }

    #[derive(Component)]
    struct LaunchCost { fuel_required: f32 }

    #[test]
    fn test_debt_increases_launch_cost() {
        let mut app = App::new();
        app.add_systems(Update, apply_debt_gravity_system);

        let planet = app.world_mut().spawn((
            PlanetaryDebt { amount: 10000.0 },
            LaunchCost { fuel_required: 100.0 },
        )).id();

        app.update();

        let cost = app.world().get::<LaunchCost>(planet).unwrap();
        assert!(cost.fuel_required > 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn apply_debt_gravity_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Extract the calculation multiplier into a standalone, testable function.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] High debt scales launch costs correctly

## 7. Technical Guidance

- Integrate tightly with ship launch logic, possibly intervening via a Launch Request event.

## 8. Questions
*Builder: add questions here if spec is unclear.*
