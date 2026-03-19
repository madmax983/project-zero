
# Specification: The Galactic Market

## 1. Overview
**Layer:** 3
**Fantasy:** The invisible hand of the market is an iron fist.
**Mechanic:** A galaxy-wide resource market where prices fluctuate based on supply and demand from all civilizations. Flooding the market crashes the price.
**Emergence:** You fund your war by selling massive amounts of food, causing a price crash that starves your enemy's economy (who relied on food exports).
**Tension:** Sell now for quick cash, or hoard to drive prices up?

## 2. Dependencies
- Trade system (`src/layer2/trade.rs` or `src/layer3/diplomacy.rs`)
- Resource/Economy system

## 3. RED Phase: Tests First

```rust
use bevy_ecs::prelude::*;

#[test]
fn test_galactic_market_price_fluctuation() {
    let mut world = World::new();
    // Arrange: Initialize GalacticMarket resource with base prices.
    world.insert_resource(GalacticMarket {
        prices: vec![(ResourceType::Food, 10.0)].into_iter().collect(),
        supply_pool: vec![(ResourceType::Food, 1000.0)].into_iter().collect(),
    });

    // Act: Simulate a massive sell order for a specific resource (e.g., Food).
    // execute_market_sell(&mut world, ResourceType::Food, 5000.0);
    // update_market_prices_system(&mut world);

    // Assert: The market price for Food decreases proportionally.
    // let market = world.resource::<GalacticMarket>();
    // let new_price = market.prices.get(&ResourceType::Food).unwrap();
    // assert!(*new_price < 10.0, "Massive sell order should crash the price of Food.");
}

#[test]
fn test_market_buy_increases_price() {
    let mut world = World::new();
    // Arrange: Initialize GalacticMarket resource.
    world.insert_resource(GalacticMarket {
        prices: vec![(ResourceType::Alloys, 50.0)].into_iter().collect(),
        supply_pool: vec![(ResourceType::Alloys, 100.0)].into_iter().collect(),
    });

    // Act: Simulate a massive buy order for a resource.
    // execute_market_buy(&mut world, ResourceType::Alloys, 500.0);
    // update_market_prices_system(&mut world);

    // Assert: The market price for that resource increases.
    // let market = world.resource::<GalacticMarket>();
    // let new_price = market.prices.get(&ResourceType::Alloys).unwrap();
    // assert!(*new_price > 50.0, "Massive buy order should spike the price of Alloys.");
}

#[test]
fn test_trade_execution_uses_current_market_price() {
    let mut world = World::new();
    // Arrange: Set a specific market price for a resource.
    world.insert_resource(GalacticMarket {
        prices: vec![(ResourceType::Fuel, 25.0)].into_iter().collect(),
        supply_pool: vec![].into_iter().collect(),
    });

    // Act: Execute a trade order.
    // let credits_earned = calculate_trade_value(&world, ResourceType::Fuel, 10.0);

    // Assert: The credits exchanged exactly match the quantity * market price.
    // assert_eq!(credits_earned, 250.0, "Trade value should be quantity * dynamic market price.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// 1. Create a `GalacticMarket` resource that tracks the current price, supply pool, and demand for all tradable resources.
// 2. Add `update_market_prices_system` that adjusts prices periodically based on recent buy/sell volumes.
// 3. Integrate `GalacticMarket` into existing trade execution logic (Layer 2/3), replacing static prices with dynamic market lookups.
```

## 5. REFACTOR Phase: Quality & Design
- Implement a price floor and ceiling to prevent integer underflow/overflow or completely broken economies.
- Add a slow "normalization" mechanic where prices slowly drift back towards a baseline over time to represent wider galactic production beyond the player's immediate view.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Selling resources lowers their price.
- [ ] Buying resources raises their price.
- [ ] Trades accurately use the dynamic market price.

## 7. Technical Guidance
- The price adjustment formula doesn't need to be complex initially. A simple inverse relationship to the "recent trades pool" is sufficient for the RED/GREEN phase.
- Be mindful of float precision issues if using `f32` for prices; consider fixed-point or integers (e.g., prices in tenths of a credit) if exact transactions are required.

## 8. Questions
*Builder: add questions here if spec is unclear.*
