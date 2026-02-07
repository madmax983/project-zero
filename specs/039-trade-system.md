# 039: Trade System

## Overview

Introduces the **Trade Depot** building and the **Merchant** system. Merchants arrive periodically at the Trade Depot, offering to exchange resources (e.g., "Sell 10 Wood for 5 Metal"). This provides a critical resource sink and source, allowing players to obtain resources they cannot produce locally.

## Dependencies

- `006` — Building Placement (for `TradeDepot` building type)
- `018` — Mining & Resources (for `ColonyResources`)
- `010` — Chronicle System (for logging arrivals)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/trade_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::trade::{
        TradeDepot, MerchantState, TradeDeal, execute_trade, merchant_arrival_system
    };
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::shared::time::SimulationTime;
    use crate::layer1::chronicle::Chronicle;

    #[test]
    fn test_trade_depot_component_exists() {
        let depot = TradeDepot::default();
        // Just verify it exists and implements Default
        let _ = depot;
    }

    #[test]
    fn test_merchant_state_resource_defaults() {
        let state = MerchantState::default();
        assert!(state.active_merchant.is_none());
        assert!(state.cooldown > 0);
    }

    #[test]
    fn test_merchant_arrival_requires_depot() {
        let mut world = World::new();
        world.insert_resource(MerchantState::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(Chronicle::default());
        // No TradeDepot spawned

        merchant_arrival_system(&mut world);

        let state = world.resource::<MerchantState>();
        assert!(state.active_merchant.is_none());
    }

    #[test]
    fn test_merchant_arrival_with_depot() {
        let mut world = World::new();
        // Set cooldown to 0 to force arrival check
        world.insert_resource(MerchantState { cooldown: 0, ..Default::default() });
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });
        world.insert_resource(Chronicle::default());

        // Spawn Trade Depot
        world.spawn((
            Building { building_type: BuildingType::TradeDepot },
            TradeDepot::default(),
        ));

        merchant_arrival_system(&mut world);

        let state = world.resource::<MerchantState>();
        assert!(state.active_merchant.is_some());

        let merchant = state.active_merchant.as_ref().unwrap();
        assert!(merchant.departure_tick > 100);
        assert!(!merchant.deals.is_empty());
    }

    #[test]
    fn test_execute_trade_success() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 50.0;
        resources.metal = 0.0;
        world.insert_resource(resources);

        let deal = TradeDeal {
            cost_resource: ResourceType::Wood,
            cost_amount: 10.0,
            give_resource: ResourceType::Metal,
            give_amount: 5.0,
        };

        let success = execute_trade(&mut world, &deal);

        assert!(success);
        let res = world.resource::<ColonyResources>();
        assert!((res.wood - 40.0).abs() < f32::EPSILON);
        assert!((res.metal - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_execute_trade_insufficient_funds() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 5.0; // Need 10
        world.insert_resource(resources);

        let deal = TradeDeal {
            cost_resource: ResourceType::Wood,
            cost_amount: 10.0,
            give_resource: ResourceType::Metal,
            give_amount: 5.0,
        };

        let success = execute_trade(&mut world, &deal);

        assert!(!success);
        let res = world.resource::<ColonyResources>();
        assert!((res.wood - 5.0).abs() < f32::EPSILON);
        assert!((res.metal - 0.0).abs() < f32::EPSILON);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType`

Add `TradeDepot` to `src/layer1/building.rs`.

```rust
pub enum BuildingType {
    // ...
    TradeDepot,
}

impl BuildingType {
    pub const fn char(&self) -> char {
        match self {
            Self::TradeDepot => '$',
            // ...
        }
    }

    pub const fn cost(&self) -> ColonyResources {
        match self {
            Self::TradeDepot => ColonyResources {
                wood: 50.0,
                stone: 20.0,
                ..ColonyResources::zeroed()
            },
            // ...
        }
    }
}
```

### 2. Create `src/layer1/trade.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::building::{Building, BuildingType};
use crate::shared::time::SimulationTime;
use crate::layer1::chronicle::{Chronicle, EventImportance};
use rand::Rng;

#[derive(Component, Default)]
pub struct TradeDepot;

#[derive(Debug, Clone)]
pub struct TradeDeal {
    pub cost_resource: ResourceType,
    pub cost_amount: f32,
    pub give_resource: ResourceType,
    pub give_amount: f32,
}

#[derive(Debug, Clone)]
pub struct Merchant {
    pub name: String,
    pub arrival_tick: u64,
    pub departure_tick: u64,
    pub deals: Vec<TradeDeal>,
}

#[derive(Resource, Default)]
pub struct MerchantState {
    pub active_merchant: Option<Merchant>,
    pub cooldown: u64, // Ticks until next possible arrival
}

pub fn merchant_arrival_system(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;

    // Check if depot exists
    let depot_exists = world.query::<(&Building, &TradeDepot)>()
        .iter(world)
        .any(|(b, _)| b.building_type == BuildingType::TradeDepot); // Redundant check if component ensures type, but safe

    if !depot_exists {
        return;
    }

    let mut state = world.resource_mut::<MerchantState>();

    // Departure check
    if let Some(merchant) = &state.active_merchant {
        if current_tick >= merchant.departure_tick {
            // Log departure
            if let Some(mut chronicle) = world.get_resource_mut::<Chronicle>() {
                chronicle.add_event(current_tick, format!("Merchant {} has departed.", merchant.name), EventImportance::Standard);
            }
            state.active_merchant = None;
            state.cooldown = current_tick + 2000; // Wait a while
        }
        return;
    }

    // Arrival check
    if current_tick < state.cooldown {
        return;
    }

    // Attempt spawn (random chance per tick, or guaranteed after cooldown?)
    // For MVP, guaranteed after cooldown for predictability.

    let merchant = Merchant {
        name: "Wanderer".to_string(), // Randomize later
        arrival_tick: current_tick,
        departure_tick: current_tick + 500, // Stay for 500 ticks
        deals: vec![
            TradeDeal {
                cost_resource: ResourceType::Wood,
                cost_amount: 10.0,
                give_resource: ResourceType::Metal,
                give_amount: 5.0,
            }
            // Generate random deals later
        ],
    };

    state.active_merchant = Some(merchant.clone());

    if let Some(mut chronicle) = world.get_resource_mut::<Chronicle>() {
        chronicle.add_event(current_tick, format!("Merchant {} has arrived.", merchant.name), EventImportance::Major);
    }
}

pub fn execute_trade(world: &mut World, deal: &TradeDeal) -> bool {
    let mut resources = world.resource_mut::<ColonyResources>();

    // Check affordability
    let affordable = match deal.cost_resource {
        ResourceType::Wood => resources.wood >= deal.cost_amount,
        ResourceType::Stone => resources.stone >= deal.cost_amount,
        ResourceType::Food => resources.food >= deal.cost_amount,
        ResourceType::Metal => resources.metal >= deal.cost_amount,
        // ... handle others
        _ => false,
    };

    if !affordable {
        return false;
    }

    // Deduct
    match deal.cost_resource {
        ResourceType::Wood => resources.wood -= deal.cost_amount,
        ResourceType::Stone => resources.stone -= deal.cost_amount,
        ResourceType::Food => resources.food -= deal.cost_amount,
        ResourceType::Metal => resources.metal -= deal.cost_amount,
        _ => {},
    }

    // Add
    match deal.give_resource {
        ResourceType::Wood => resources.add_wood(deal.give_amount),
        ResourceType::Stone => resources.add_stone(deal.give_amount),
        ResourceType::Food => resources.add_food(deal.give_amount),
        ResourceType::Metal => resources.add_metal(deal.give_amount),
        _ => {},
    }

    true
}
```

### 3. Register System

In `src/layer1/mod.rs` and `main.rs`, register `MerchantState` resource and `merchant_arrival_system`.

## REFACTOR Phase: Quality & Design

- **Deal Generation**: Move deal generation to a separate function `generate_deals()`. Use randomness based on colony needs (sell what they lack) or random.
- **Names**: Use `NarrativeGenerator` to name merchants.
- **UI**: Add a simple debug UI or integrate with `Status` bar to show "Merchant Arrived [T]rade". (Future UI spec).
- **Resource Helper**: `ColonyResources` should have `get(ResourceType)` and `remove(ResourceType, amount)` to avoid the match statements.

## Acceptance Criteria

- [ ] `TradeDepot` building is constructible.
- [ ] `Merchant` arrives only if `TradeDepot` exists.
- [ ] `Merchant` departs after a duration.
- [ ] `Chronicle` logs arrivals/departures.
- [ ] `execute_trade` correctly swaps resources.
- [ ] Tests pass.

## Technical Guidance

- Use `rand::thread_rng()` for probability.
- Ensure `ResourceType` matches allow for all resource types.
- `MerchantState` should persist.

## Questions

*Builder: add questions here if spec is unclear.*
