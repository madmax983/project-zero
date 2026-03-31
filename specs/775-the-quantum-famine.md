# 775: The Quantum Famine

## 1. Overview
**Layer:** Cross-layer (3 -> 2 -> 1)
**Fantasy:** The galaxy's economy is so interconnected that the mere prediction of a shortage causes actual starvation today.
**Mechanic:** High-level market predictions of future food scarcity cause automated trade fleets to preemptively hoard food. Layer 1 colonies with open trade instantly experience artificial food shortages because local suppliers sell everything off-world at speculative markups.

## 2. Dependencies
- Layer 3 Economy/Market Prediction
- Layer 1 Trade/Logistics logic

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_market_panic_triggers_local_export_dump() {
        let mut app = App::new();
        app.add_event::<MarketPanicEvent>();
        app.add_systems(Update, process_market_panic_hoarding);

        let local_farm_entity = app.world_mut().spawn((
            LocalStockpile { food: 500.0 },
            TradePolicy { open: true },
        )).id();

        app.world_mut().send_event(MarketPanicEvent {
            commodity: Commodity::Food,
            severity_multiplier: 5.0,
        });

        app.update();

        // Local stock should be depleted because trade policy is open and panic triggered an export
        let stock = app.world().get::<LocalStockpile>(local_farm_entity).unwrap();
        assert!(stock.food < 100.0, "Stockpile should be nearly emptied due to panic selling");
    }

    #[test]
    fn test_closed_trade_ignores_market_panic() {
        let mut app = App::new();
        app.add_event::<MarketPanicEvent>();
        app.add_systems(Update, process_market_panic_hoarding);

        let local_farm_entity = app.world_mut().spawn((
            LocalStockpile { food: 500.0 },
            TradePolicy { open: false }, // Closed borders
        )).id();

        app.world_mut().send_event(MarketPanicEvent {
            commodity: Commodity::Food,
            severity_multiplier: 5.0,
        });

        app.update();

        // Stock remains untouched because they didn't export
        let stock = app.world().get::<LocalStockpile>(local_farm_entity).unwrap();
        assert_eq!(stock.food, 500.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct LocalStockpile {
    pub food: f32,
}

#[derive(Component)]
pub struct TradePolicy {
    pub open: bool,
}

#[derive(PartialEq)]
pub enum Commodity {
    Food,
    Alloys,
    Energy,
}

#[derive(Event)]
pub struct MarketPanicEvent {
    pub commodity: Commodity,
    pub severity_multiplier: f32,
}

pub fn process_market_panic_hoarding(
    mut events: EventReader<MarketPanicEvent>,
    mut stockpiles: Query<(&mut LocalStockpile, &TradePolicy)>,
) {
    for event in events.read() {
        if event.commodity == Commodity::Food {
            for (mut stock, policy) in stockpiles.iter_mut() {
                if policy.open {
                    // Sell off 90% of stock to passing speculative freighters
                    let sell_amount = stock.food * 0.90;
                    stock.food -= sell_amount;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: Make the logic generic so it works for all `Commodities` by having a `HashMap<Commodity, f32>` in the stockpile instead of a hardcoded `food: f32` field.
- **Performance**: Event-driven architecture means the logic only runs when a panic event is received.
- **API Improvements**: The system should generate an event (e.g., `ExportDumpEvent`) so that off-world ships or planetary credit accounts can actually receive the money for the dumped food.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Open trade dumps food on panic, closed trade protects food.

## 7. Technical Guidance
- The trigger for `MarketPanicEvent` should live in a Layer 3 system predicting future scarcity based on war or drought across nodes.
- When food is dumped, the planet should receive massive credits (to represent the speculative markups), but the population will immediately face starvation next tick.

## 8. Questions
*Builder: add questions here if spec is unclear.*
