# 923 - The Black Market Airlock

## 1. Overview

**Layer:** 1
**Fantasy:** Contraband entering the colony outside official channels, creating an illicit economy.
**Mechanic:** Pops with low morale secretly repurpose a remote or broken airlock to smuggle restricted goods. This creates an underground economy that boosts morale but bypasses colony storage and taxation.

## 2. Dependencies

- Layer 1 Needs (`Morale`/`UtilityWeights`)
- Infrastructure tracking (Airlocks/Doors)
- Inventory systems

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct Airlock { is_broken: bool }

    #[derive(Component)]
    struct BlackMarketNode { contraband_volume: f32 }

    #[derive(Component)]
    struct PopMorale { value: f32 }

    #[test]
    fn test_broken_airlock_becomes_black_market() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_black_market_formation_system);

        let airlock = app.world_mut().spawn((
            Airlock { is_broken: true },
        )).id();

        // Spawn a pop with low morale nearby
        let pop = app.world_mut().spawn((
            PopMorale { value: 10.0 },
            Transform::from_xyz(1.0, 0.0, 0.0), // Assuming airlock is at origin
        )).id();

        app.update();

        // Black market should form
        assert!(app.world().get::<BlackMarketNode>(airlock).is_some());
    }

    #[test]
    fn test_black_market_boosts_morale() {
        let mut app = App::new();
        app.add_systems(Update, process_black_market_effects_system);

        let airlock = app.world_mut().spawn((
            BlackMarketNode { contraband_volume: 50.0 },
        )).id();

        let pop = app.world_mut().spawn((
            PopMorale { value: 10.0 },
        )).id();

        app.update();

        let morale = app.world().get::<PopMorale>(pop).unwrap();
        assert!(morale.value > 10.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn evaluate_black_market_formation_system(...) { ... }
// pub fn process_black_market_effects_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Ensure `BlackMarketNode` generation uses a timer or probability rather than spawning instantly on the first tick.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] Black market formation requires a broken airlock and low morale pops
- [ ] Morale is correctly boosted by the black market

## 7. Technical Guidance

- Proximity checks can be heavy; use spatial partitioning if available.

## 8. Questions
*Builder: add questions here if spec is unclear.*
