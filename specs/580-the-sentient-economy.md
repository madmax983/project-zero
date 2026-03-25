# 580: The Sentient Economy

## 1. Overview
The Layer 3 galactic market actively learns the player's trading patterns. If the player repeatedly exploits a specific resource loop (e.g., mass-producing cheap nutrient paste), the market AI artificially crashes the price of that good and inflates the price of its prerequisites out of spite, demanding economic novelty. This punishes perfectly optimized monopolies.

## 2. Dependencies
- Core Layer 3 ECS (Entities, Components, Systems)
- Trade Resource system (`TradeResource`, `SellAction`)
- Market Price history (`GalacticMarket`)
- Price calculation modifiers

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::trade::{GalacticMarket, SellAction, TradeResource};
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<SellAction>();
        app.add_systems(Update, process_sentient_market_system);
        app
    }

    #[test]
    fn test_market_tolerates_varied_trading() {
        let mut app = setup_app();

        // Arrange: A market
        app.world.insert_resource(GalacticMarket {
            prices: vec![(TradeResource::Uranium, 100)].into_iter().collect(),
            history: MarketHistory::default(),
        });

        // Act: Sell Uranium a few times (under the spite threshold)
        app.world.send_event(SellAction { resource: TradeResource::Uranium, amount: 10 });
        app.update();

        // Assert: Price remains stable, no spite crash
        let market = app.world.resource::<GalacticMarket>();
        assert_eq!(market.prices.get(&TradeResource::Uranium).unwrap(), &100);
    }

    #[test]
    fn test_market_spite_crashes_monopolies() {
        let mut app = setup_app();

        // Arrange: A market and history tracking
        app.world.insert_resource(GalacticMarket {
            prices: vec![(TradeResource::Uranium, 100)].into_iter().collect(),
            history: MarketHistory {
                consecutive_sales: vec![(TradeResource::Uranium, 5)].into_iter().collect(), // Threshold reached
            },
        });

        // Act: Sell Uranium again
        app.world.send_event(SellAction { resource: TradeResource::Uranium, amount: 10 });
        app.update();

        // Assert: The market AI spikes the price to near zero out of spite
        let market = app.world.resource::<GalacticMarket>();
        assert!(market.prices.get(&TradeResource::Uranium).unwrap() < &10);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum TradeResource {
    Uranium,
    NutrientPaste,
}

#[derive(Resource, Default)]
pub struct GalacticMarket {
    pub prices: HashMap<TradeResource, i32>,
    pub history: MarketHistory,
}

#[derive(Default)]
pub struct MarketHistory {
    pub consecutive_sales: HashMap<TradeResource, u32>,
}

#[derive(Event)]
pub struct SellAction {
    pub resource: TradeResource,
    pub amount: u32,
}

pub fn process_sentient_market_system(
    mut events: EventReader<SellAction>,
    mut market: ResMut<GalacticMarket>,
) {
    for event in events.read() {
        let count = market.history.consecutive_sales.entry(event.resource).or_insert(0);
        *count += 1;

        if *count > 5 {
            if let Some(price) = market.prices.get_mut(&event.resource) {
                *price = 1; // Artificial spite crash
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Spite crash logic hardcoded to `1`. Should apply a massive negative modifier (`SpiteCrashModifier`) to integrate with dynamic pricing instead of overriding it directly.
- **Performance**: Standard event processing, fast.
- **API Improvements**: The `MarketHistory` should track sales over time, allowing the player to let the "spite" cool down. Introduce a decay function to `consecutive_sales`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Over-selling a single resource triggers an artificial price crash in the `GalacticMarket`.
- [ ] The crash functions independently of standard supply/demand shifts (it's actively punitive).

## 7. Technical Guidance
- Enhance the `update_market_prices_system` to recognize and read the `MarketHistory` spite counters.
- Inflate the prerequisites (e.g., if you mass sell Nutrient Paste, the price of Water or Biomass should artificially skyrocket to punish you).

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
