# 436: The Debt Collector's Blockade

## Overview

If the colony accumulates too much debt with a Layer 3 megacorp, they deploy a massive, indestructible "Collection Sphere" enveloping the entire Layer 2 system. It intercepts all incoming trade and migrant ships, siphoning their cargo to pay your debt, effectively isolating the colony from the rest of the galaxy until the debt is paid off.

## Dependencies

- `039` Trade System
- `152` Orbital Stations

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ColonyDebt>();
        app.add_event::<TradeShipArrivalEvent>();
        app.add_systems(Update, (debt_blockade_system, blockade_interception_system));
        app
    }

    #[test]
    fn test_blockade_spawns_when_debt_critical() {
        let mut app = setup_test_app();

        app.world.resource_mut::<ColonyDebt>().amount = 100_000.0;
        app.world.resource_mut::<ColonyDebt>().threshold = 50_000.0;

        app.update();

        let blockades = app.world.query::<&CollectionSphereBlockade>().iter(&app.world).count();
        assert_eq!(blockades, 1, "A Collection Sphere should spawn when debt exceeds threshold");
    }

    #[test]
    fn test_blockade_intercepts_trade_ships() {
        let mut app = setup_test_app();

        // Setup blockade
        app.world.spawn(CollectionSphereBlockade { active: true });

        // Setup a pending trade ship arrival
        app.world.send_event(TradeShipArrivalEvent {
            cargo_value: 5000.0,
            faction: "Megacorp".to_string(),
        });

        // Debt before interception
        app.world.resource_mut::<ColonyDebt>().amount = 50_000.0;

        app.update();

        // The event should have been intercepted, reducing debt instead of delivering cargo
        let debt = app.world.resource::<ColonyDebt>().amount;
        assert_eq!(debt, 45_000.0, "The trade ship's cargo should be siphoned to pay debt");

        // Verify no actual cargo was added to colony (assuming ColonyResources is checked)
    }

    #[test]
    fn test_blockade_lifts_when_debt_paid() {
        let mut app = setup_test_app();

        // Setup blockade and zero debt
        let blockade_entity = app.world.spawn(CollectionSphereBlockade { active: true }).id();
        app.world.resource_mut::<ColonyDebt>().amount = 0.0;
        app.world.resource_mut::<ColonyDebt>().threshold = 50_000.0;

        app.update();

        // The blockade should despawn or deactivate
        assert!(app.world.get::<CollectionSphereBlockade>(blockade_entity).is_none(), "Blockade should despawn when debt is cleared");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ColonyDebt {
    pub amount: f32,
    pub threshold: f32,
}

#[derive(Component)]
pub struct CollectionSphereBlockade {
    pub active: bool,
}

#[derive(Event)]
pub struct TradeShipArrivalEvent {
    pub cargo_value: f32,
    pub faction: String,
}

pub fn debt_blockade_system(
    mut commands: Commands,
    debt: Res<ColonyDebt>,
    query: Query<Entity, With<CollectionSphereBlockade>>,
) {
    let is_critical = debt.amount >= debt.threshold;
    let has_blockade = !query.is_empty();

    if is_critical && !has_blockade {
        commands.spawn(CollectionSphereBlockade { active: true });
    } else if !is_critical && has_blockade {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn blockade_interception_system(
    mut debt: ResMut<ColonyDebt>,
    blockade_query: Query<&CollectionSphereBlockade>,
    mut trade_events: EventReader<TradeShipArrivalEvent>,
) {
    if blockade_query.is_empty() {
        return; // No blockade, ships pass normally
    }

    for event in trade_events.read() {
        // Intercept ship cargo to pay debt
        debt.amount -= event.cargo_value;
        if debt.amount < 0.0 {
            debt.amount = 0.0;
        }
        // In a real system, we'd also prevent the cargo from reaching the colony here
    }
}
```

## REFACTOR Phase: Quality & Design

- Introduce a `TradeInterceptionEvent` to notify the player that their shipments were seized by the blockade.
- Make the `CollectionSphereBlockade` a visual entity on Layer 2 (orbit) so the player can actually see the sphere enveloping the planet.
- Calculate dynamic thresholds based on the colony's net worth or galactic standing instead of a static value.
- Add negative mood/unrest modifiers for Pops when the colony is under blockade.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] When debt threshold is reached, a `CollectionSphereBlockade` spawns.
- [ ] Active blockade intercepts incoming `TradeShipArrivalEvent`s, applying cargo value to debt.
- [ ] Blockade despawns when debt is completely paid.

## Technical Guidance

- Ensure the `blockade_interception_system` runs *before* whatever system normally processes `TradeShipArrivalEvent` and delivers resources to the colony, so it can consume/intercept the events effectively.
- For Bevy event interception, since Bevy events are broadcast to all readers, the interception system might need to consume the event and emit a different `DeliveredTradeEvent` if no blockade exists, rather than trying to "stop" the event.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
