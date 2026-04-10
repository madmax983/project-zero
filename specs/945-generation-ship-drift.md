# 945: Generation Ship Drift

## Overview

When launching sub-light Generation Ships, the colony inside the ship simulates years of cultural and social drift during the transit based on onboard conditions. When the ship finally arrives and establishes the Layer 1 colony, it inherits the result of that drift, potentially mutating into a completely alien or hostile civilization depending on the hardships faced in transit.

## Dependencies

- Requires Layer 3 Fleet systems and bridging to Layer 1 colony initialization.

## 1. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer3::fleets::{Fleet, Destination};
    use crate::layer1::social::Culture;

    #[test]
    fn test_transit_drift_accumulation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, simulate_transit_drift_system);

        let ship_id = app.world_mut().spawn((
            GenerationShip,
            TransitConditions { food_scarcity: true, mechanical_failures: 2 },
            TransitDrift { hostility_score: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert
        // Hostility score should increase due to poor conditions
        let drift = app.world().get::<TransitDrift>(ship_id).unwrap();
        assert!(drift.hostility_score > 0.0, "Drift hostility should increase under poor conditions.");
    }

    #[test]
    fn test_colony_foundation_with_drift() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ColonyFoundedEvent>();
        app.add_systems(Update, apply_drift_on_foundation_system);

        let ship_id = app.world_mut().spawn((
            GenerationShip,
            TransitDrift { hostility_score: 100.0 }, // High hostility
        )).id();

        let target_planet_id = app.world_mut().spawn_empty().id();

        // Act
        app.world_mut().send_event(ColonyFoundedEvent {
            source_ship: ship_id,
            target_planet: target_planet_id,
        });
        app.update();

        // Assert
        // Target planet should now have a hostile culture
        let culture = app.world().get::<Culture>(target_planet_id).unwrap();
        assert_eq!(culture.alignment, Alignment::Hostile, "High drift should result in hostile alignment.");
    }
}
```

## 2. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Stub structures for dependencies
#[derive(Component)]
pub struct GenerationShip;

#[derive(Component, Default)]
pub struct TransitConditions {
    pub food_scarcity: bool,
    pub mechanical_failures: u32,
}

#[derive(Event)]
pub struct ColonyFoundedEvent {
    pub source_ship: Entity,
    pub target_planet: Entity,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Alignment {
    Peaceful,
    Hostile,
}

#[derive(Component)]
pub struct Culture {
    pub alignment: Alignment,
}

// Feature components
#[derive(Component)]
pub struct TransitDrift {
    pub hostility_score: f32,
}

// Systems
pub fn simulate_transit_drift_system(
    mut ships: Query<(&TransitConditions, &mut TransitDrift), With<GenerationShip>>,
) {
    for (conditions, mut drift) in ships.iter_mut() {
        if conditions.food_scarcity {
            drift.hostility_score += 10.0;
        }
        drift.hostility_score += conditions.mechanical_failures as f32 * 5.0;
    }
}

pub fn apply_drift_on_foundation_system(
    mut commands: Commands,
    mut events: EventReader<ColonyFoundedEvent>,
    ships: Query<&TransitDrift, With<GenerationShip>>,
) {
    for event in events.read() {
        if let Ok(drift) = ships.get(event.source_ship) {
            let alignment = if drift.hostility_score > 50.0 {
                Alignment::Hostile
            } else {
                Alignment::Peaceful
            };

            commands.entity(event.target_planet).insert(Culture { alignment });
        }
    }
}
```

## 3. REFACTOR Phase: Quality & Design

- `TransitConditions` should realistically be updated by other sub-systems or events during the transit rather than being static values.
- Expand `Culture` to support more granular drift attributes (e.g., technophobia, fanaticism) rather than a simple `Alignment` enum.
- Consider integrating this with the Chronicle system so the player can read logs of *how* the ship degenerated during transit.

## 4. Acceptance Criteria (Testable)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] Transit conditions actively impact the `TransitDrift` of a generation ship over time.
- [ ] The drifted traits are correctly applied to the new colony upon arrival.

## 5. Technical Guidance

- This is a cross-layer feature. `simulate_transit_drift_system` likely runs in Layer 3 (or Layer 2 fleet logic), while `apply_drift_on_foundation_system` acts as a bridge to Layer 1 initialization.
- Coordinate with `src/shared/narrative.rs` if integrating Chronicle events for the journey's descent into madness.

## 6. Questions

*Builder: add questions here if spec is unclear.*
