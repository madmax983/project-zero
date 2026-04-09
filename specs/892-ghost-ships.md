# 892 - Ghost Ships

## 1. Overview
**What:** Ships traveling between stars have a small chance of "incidents" resulting in them being lost. Sometimes, these lost ships return years later as "Ghost Ships" with altered cargo, changed crews, or mysterious warnings.
**Why:** To emphasize the danger of interstellar travel and create narrative tension. A lost ship is a tragedy; a returning ship is a mystery that forces the player to decide whether to accept the potentially dangerous unknown.

## 2. Dependencies
- Layer 3 Core (Stellar Cartography, Fleet Navigation)
- A Time tracking system (e.g., `Chronicle` or Bevy's `Time`) to measure the duration a ship is lost.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ship_becomes_lost_during_transit() {
        let mut app = App::new();
        // Setup systems
        // ...

        let ship_entity = app.world_mut().spawn((
            Ship,
            TransitRoute { hazard_level: 0.8 }, // High hazard route
        )).id();

        // Trigger the incident evaluation system
        app.world_mut().send_event(EvaluateTransitEvent { ship: ship_entity });
        app.update(); // Assume RNG favors an incident for the test

        // The ship should now have the LostInTransit component
        let lost_comp = app.world().get::<LostInTransit>(ship_entity);
        assert!(lost_comp.is_some());
    }

    #[test]
    fn test_ghost_ship_returns() {
        let mut app = App::new();
        // Setup systems
        // ...

        let ship_entity = app.world_mut().spawn((
            Ship,
            LostInTransit { cycles_lost: 10 },
        )).id();

        // Trigger the return evaluation system
        app.world_mut().send_event(EvaluateLostShipReturnEvent { ship: ship_entity });
        app.update(); // Assume RNG favors return for the test

        // The ship should lose LostInTransit and gain GhostShip
        assert!(app.world().get::<LostInTransit>(ship_entity).is_none());
        assert!(app.world().get::<GhostShip>(ship_entity).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng; // Depending on what the project uses for RNG

#[derive(Component)]
pub struct LostInTransit {
    pub cycles_lost: u32,
}

#[derive(Component)]
pub struct GhostShip {
    pub anomaly_type: String, // Or an enum for cargo changes, crew changes, etc.
}

#[derive(Event)]
pub struct EvaluateTransitEvent {
    pub ship: Entity,
}

#[derive(Event)]
pub struct EvaluateLostShipReturnEvent {
    pub ship: Entity,
}

// ... Systems to handle the events, applying RNG based on hazard levels,
// and transitioning components from Normal -> LostInTransit -> GhostShip
```

## 5. REFACTOR Phase: Quality & Design
- Centralize RNG usage to ensure it can be mocked or seeded for more reliable testing without relying on "assuming RNG favors" comments.
- Expand `GhostShip` anomaly types into a robust enum rather than a simple string.
- Integrate the return of a Ghost Ship into the Chronicle system to record the lore event.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Ships can correctly transition from normal transit to lost, and from lost to a returned ghost ship.

## 7. Technical Guidance
- Be mindful of how a `LostInTransit` ship is handled by other systems; it should probably be excluded from normal collision or fleet management queries until it returns.
- Consider utilizing the `Delay` or timer components if the return mechanic is purely time-based rather than RNG-based each cycle.

## 8. Questions
*Builder: add questions here if spec is unclear.*
