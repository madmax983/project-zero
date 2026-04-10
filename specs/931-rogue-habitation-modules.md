# 931 - Rogue Habitation Modules

## 1. Overview

**Layer:** 2
**Fantasy:** A neighborhood simply deciding to leave the planet.
**Mechanic:** High-tier orbital housing units have emergency thrusters. If the inhabitants' unrest reaches critical mass, instead of rioting, they fire the thrusters, decouple from the main station or colony, and become an independent, mobile mini-station.
**Emergence:** You overtax your wealthiest orbital citizens. Instead of paying, they launch their luxury condos into deep space, taking a massive chunk of your tax base and several vital trade delegates with them.
**Tension:** Keeping powerful pops happy not just to avoid riots, but to physically prevent them from taking their infrastructure and leaving.

## 2. Dependencies

- Layer 2 / Orbital Station mechanics
- Pop Unrest / Morale tracking per module
- Economy / Tax logic
- Entity hierarchy (modules attached to stations)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct OrbitalHousingModule { tier: u32, is_attached: bool }

    #[derive(Component)]
    struct Unrest { level: f32, threshold: f32 }

    #[derive(Component)]
    struct MobileStation;

    #[test]
    fn test_high_unrest_triggers_module_decoupling() {
        let mut app = App::new();
        app.add_systems(Update, process_rogue_module_system);

        let module = app.world_mut().spawn((
            OrbitalHousingModule { tier: 3, is_attached: true }, // High tier
            Unrest { level: 90.0, threshold: 80.0 }, // Past threshold
        )).id();

        app.update();

        // The module should detach and become its own station
        let updated_module = app.world().get::<OrbitalHousingModule>(module).unwrap();
        assert!(!updated_module.is_attached);
        assert!(app.world().get::<MobileStation>(module).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn process_rogue_module_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- The decoupling process should remove the entity from its parent `Children` hierarchy if utilizing Bevy's spatial hierarchy.
- Convert the module to a new type of floating entity that requires its own pathfinding/orbit logic.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] High-tier housing modules with unrest above threshold detach from their station.

## 7. Technical Guidance

- Use an event (e.g., `ModuleDecoupledEvent`) to notify other systems (like the economy) that the tax base has been lost.
- Ensure only "High-tier" housing (e.g., tier >= 3) has this capability; low-tier slums just riot.

## 8. Questions
*Builder: add questions here if spec is unclear.*
