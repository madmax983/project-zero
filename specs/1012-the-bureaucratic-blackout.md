# 1012: The Bureaucratic Blackout

## 1. Overview
A severe glitch or cyber-attack on Layer 3 deletes a colony's "Imperial Charter." Suddenly, Layer 1 receives no trade ships, no communications, and no tax collectors. The colony is functionally isolated, creating an immediate boom of freedom and black-market efficiency. However, when the Empire inevitably rediscovers the colony decades later, it demands all back-taxes instantly, prompting a violent rebellion from Pops who don't even remember the Empire.

## 2. Dependencies
- Layer 1/2 `Colony` and `Economy` tracking.
- Layer 3 `Diplomacy` (Taxes, Imperial relations).
- Layer 2 `Trade` ship routing.
- `Unrest` and `Rebellion` mechanics.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::colony::{Colony, ImperialCharter, BackTaxes};
    use crate::layer2::trade::TradeRoute;
    use crate::layer1::diplomacy::{TaxCollectionEvent, RebellionEvent};

    #[test]
    fn test_charter_deletion_severs_trade_routes() {
        let mut app = App::new();
        app.add_event::<CharterDeletedEvent>();
        app.add_systems(Update, isolate_colony_system);

        let colony = app.world_mut().spawn((
            Colony,
            ImperialCharter { active: true },
        )).id();

        let trade_route = app.world_mut().spawn((
            TradeRoute { destination: colony },
        )).id();

        app.world_mut().resource_mut::<Events<CharterDeletedEvent>>().send(CharterDeletedEvent {
            colony,
        });

        app.update();

        // Verify charter is inactive
        let charter = app.world().get::<ImperialCharter>(colony).unwrap();
        assert!(!charter.active, "Imperial Charter should be marked inactive.");

        // Verify trade route is destroyed/disabled
        assert!(app.world().get_entity(trade_route).is_err(), "Trade routes to an isolated colony should be severed.");
    }

    #[test]
    fn test_tax_collection_after_blackout_triggers_rebellion() {
        let mut app = App::new();
        app.add_event::<TaxCollectionEvent>();
        app.add_event::<RebellionEvent>();
        app.add_systems(Update, process_back_taxes_system);

        let colony = app.world_mut().spawn((
            Colony,
            BackTaxes { amount_owed: 50000, years_accrued: 50 }, // Massive debt
            ImperialCharter { active: true }, // Re-discovered
        )).id();

        app.world_mut().resource_mut::<Events<TaxCollectionEvent>>().send(TaxCollectionEvent {
            colony,
            amount: 50000,
        });

        app.update();

        let rebellion_events = app.world().resource::<Events<RebellionEvent>>();
        let mut reader = rebellion_events.get_reader();
        let mut found = false;
        for event in reader.read(rebellion_events) {
            if event.colony == colony {
                found = true;
            }
        }

        assert!(found, "Attempting to collect massive multi-generational back taxes should trigger a rebellion.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer3/bureaucratic_blackout.rs
use bevy::prelude::*;
use crate::layer1::colony::{Colony, ImperialCharter, BackTaxes};
use crate::layer2::trade::TradeRoute;
use crate::layer1::diplomacy::{TaxCollectionEvent, RebellionEvent};

#[derive(Event)]
pub struct CharterDeletedEvent {
    pub colony: Entity,
}

pub fn isolate_colony_system(
    mut commands: Commands,
    mut events: EventReader<CharterDeletedEvent>,
    mut colony_query: Query<&mut ImperialCharter>,
    trade_route_query: Query<(Entity, &TradeRoute)>,
) {
    for event in events.read() {
        if let Ok(mut charter) = colony_query.get_mut(event.colony) {
            charter.active = false;
        }

        for (route_ent, route) in trade_route_query.iter() {
            if route.destination == event.colony {
                commands.entity(route_ent).despawn();
            }
        }
    }
}

pub fn process_back_taxes_system(
    query: Query<(&ImperialCharter, &BackTaxes)>,
    mut tax_events: EventReader<TaxCollectionEvent>,
    mut rebellion_events: EventWriter<RebellionEvent>,
) {
    for event in tax_events.read() {
        if let Ok((charter, back_taxes)) = query.get(event.colony) {
            if charter.active && back_taxes.years_accrued >= 30 {
                // Generational disconnect causes instant rebellion instead of payment
                rebellion_events.send(RebellionEvent {
                    colony: event.colony,
                    reason: "Generational Tax Disconnect".to_string(),
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Economy Boom:** While the `ImperialCharter` is inactive, the colony should receive a persistent buff to its local economy or shadow-market efficiency (no taxes being drained).
- **Accrual System:** A system needs to continuously increase `BackTaxes.amount_owed` and `years_accrued` while the charter is inactive.
- **Re-discovery Trigger:** There should be a specific event or criteria for the Empire to "find" the colony again (e.g., building a massive communications array or Layer 2 station).

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_charter_deletion_severs_trade_routes` passes.
- [ ] Test `test_tax_collection_after_blackout_triggers_rebellion` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Despawning `TradeRoute` entities might leave orphaned `TradeShip` entities in Layer 2. Ensure your cleanup logic cascades properly.
- The threshold for a "Generational Disconnect" rebellion is arbitrarily set at 30 years; this could be tied to the average lifespan of the specific Pop species.

## 8. Questions
*Builder: add questions here if spec is unclear.*
