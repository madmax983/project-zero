# ADR 045: Galactic Market System

## Status
Proposed

## Context
The simulation lacked a dynamic economic system for trading resources across the interstellar scale (Layer 3). Prices for resources were previously static, which failed to model the consequences of overproduction or scarcity.

## Decision
Introduced a `GalacticMarket` resource in `src/layer3/market/galactic_market.rs` that dynamically tracks the `prices`, `supply_pool`, and `baseline_prices` of all tradeable resources. A new system, `update_market_prices_system`, recalculates prices each tick based on the inverse relationship between supply and baseline expectations.

## Consequences
- **Dynamic Economy:** Players can no longer exploit static prices by flooding the market with easily produced goods (like `Waste` or `Food`). Overproduction crashes the price.
- **Strategic Trading:** Scarcity drives prices up, creating opportunities for specialized colonies to profit off rare materials.
- **Architectural Boundary:** The market exists in Layer 3 (`layer3::market::galactic_market`), establishing a clear seam where Layer 1 (Colony Production) and Layer 2 (Trade Ships/Routes) intersect via the `supply_pool`.
