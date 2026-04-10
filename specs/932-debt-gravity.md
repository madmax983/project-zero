# 932 - Debt Gravity

## 1. Overview

**Layer:** Cross-layer
**Fantasy:** Economic failure physically trapping you on a world.
**Mechanic:** Pops and factions track severe financial debt. High levels of collective debt physically increase the mass/fuel cost required to launch ships from that colony, representing the bureaucratic and literal blockade of debt collectors seizing assets and fuel.
**Emergence:** A once-thriving trade hub suffers an economic crash. The massive resulting debt makes it too expensive to launch merchant fleets to trade their way out of the crash, turning the planet into an inescapable 'debt sink' where nobody can afford the fuel to leave.
**Tension:** Taking on loans to rapidly develop a colony, knowing that if you fail to pay, the planet itself will become a gravitational prison.

## 2. Dependencies

- Economy / Debt tracking systems
- Launch Cost calculation / Fleet mechanics
- Faction / Colony metadata

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct ColonyDebt { amount: f32 }

    #[derive(Component)]
    struct ShipLaunchPad { base_fuel_cost: f32, current_cost: f32 }

    #[test]
    fn test_high_debt_increases_launch_cost() {
        let mut app = App::new();
        app.add_systems(Update, calculate_debt_gravity_system);

        let colony = app.world_mut().spawn((
            ColonyDebt { amount: 500_000.0 }, // Massive debt
            ShipLaunchPad { base_fuel_cost: 1000.0, current_cost: 1000.0 },
        )).id();

        app.update();

        // Launch cost should be significantly higher due to debt gravity
        let pad = app.world().get::<ShipLaunchPad>(colony).unwrap();
        assert!(pad.current_cost > pad.base_fuel_cost);
        assert!(pad.current_cost > 2000.0); // Assuming some scaling factor
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn calculate_debt_gravity_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Ensure the `current_cost` is recalculated on every frame or when debt changes, rather than being permanently modified.
- Include a UI element or warning system that triggers when "Debt Gravity" becomes a significant factor for a colony.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Colony debt exponentially or linearly increases the fuel cost for ship launches from that colony.

## 7. Technical Guidance

- Link the debt value to the pre-existing launch calculation logic. If launch logic exists in another file, you might need an Integration System to bridge `ColonyDebt` to `ShipLaunchPad` cost.
- Be careful with exponential scaling to avoid integer/float overflows for extremely high debt values.

## 8. Questions
*Builder: add questions here if spec is unclear.*
