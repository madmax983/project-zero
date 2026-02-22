# 203: Prohibition & Contraband

## Overview

Laws are made to be broken. This feature allows the player to Ban specific resources (e.g., "No Alcohol").
Banned items become **Contraband**.
- **Contraband**: Cannot be sold legally.
- **Black Market**: Trade value for Contraband doubles.
- **Possession**: Pops holding Contraband gain "Criminal" status or risk arrest.
- **Smuggling**: Traders will still buy/sell but at higher risk/price.

## Dependencies

- `054` — Colony Edicts (UI to ban items)
- `039` — Trade System (Market values)
- `072` — Justice System (Crime events)

## RED Phase: Tests First

Write these tests in `src/layer1/contraband_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::trade::{TradeMarket, MarketItem, MarketStatus};
    use crate::layer1::resources::ResourceType;
    use crate::layer1::edicts::{EdictManager, Edict};
    use crate::layer1::contraband::{Contraband, enforce_prohibition_system};

    #[test]
    fn test_banning_resource() {
        let mut world = World::new();
        // Setup Market
        let mut market = TradeMarket::default();
        market.items.insert(ResourceType::Alcohol, MarketItem {
            base_price: 10.0,
            status: MarketStatus::Legal
        });
        world.insert_resource(market);

        // Setup Edict Manager
        let mut edicts = EdictManager::default();
        // Assume Edict::Prohibition(ResourceType) exists or we add it
        edicts.active.push(Edict::Prohibition(ResourceType::Alcohol));
        world.insert_resource(edicts);

        // Run system to sync edicts to market status
        enforce_prohibition_system(&mut world);

        let market = world.get_resource::<TradeMarket>().unwrap();
        let item = market.items.get(&ResourceType::Alcohol).unwrap();
        assert_eq!(item.status, MarketStatus::Contraband);
        // Price should be higher (e.g., 2x)
        assert_eq!(item.current_price(), 20.0);
    }

    #[test]
    fn test_possession_crime_trigger() {
        let mut world = World::new();

        // Setup Pop with Alcohol
        let pop = world.spawn((
            crate::layer1::pop::Pop,
            crate::layer1::inventory::Inventory {
                items: vec![(ResourceType::Alcohol, 1)]
            },
            // Expected component added by system
            // ContrabandPossession
        )).id();

        // Setup Market stating Alcohol is Contraband
        let mut market = TradeMarket::default();
        market.items.insert(ResourceType::Alcohol, MarketItem {
            base_price: 10.0,
            status: MarketStatus::Contraband
        });
        world.insert_resource(market);

        // Run detection system
        detect_contraband_system(&mut world);

        assert!(world.get::<ContrabandPossession>(pop).is_some());
        // Verify Crime Event triggered? (Mock event writer)
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Enums

Update `Edict` enum in `src/layer1/edicts.rs` to include `Prohibition(ResourceType)`.
Update `MarketStatus` enum in `src/layer1/trade.rs` to include `Contraband`.

### 2. Implement Systems (`src/layer1/contraband.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::trade::{TradeMarket, MarketStatus};
use crate::layer1::edicts::{EdictManager, Edict};
use crate::layer1::inventory::Inventory;

#[derive(Component)]
pub struct ContrabandPossession;

pub fn enforce_prohibition_system(
    edicts: Res<EdictManager>,
    mut market: ResMut<TradeMarket>,
) {
    // Reset all to Legal first? Or handle state carefully.
    // For MVP: iterate active edicts.

    for edict in &edicts.active {
        if let Edict::Prohibition(resource) = edict {
            if let Some(item) = market.items.get_mut(resource) {
                item.status = MarketStatus::Contraband;
                // Apply price multiplier logic in item.current_price() or here
                // item.price_multiplier = 2.0;
            }
        }
    }
}

pub fn detect_contraband_system(
    market: Res<TradeMarket>,
    mut commands: Commands,
    query: Query<(Entity, &Inventory), Without<ContrabandPossession>>,
) {
    for (entity, inventory) in query.iter() {
        let has_contraband = inventory.items.iter().any(|(res, _)| {
            market.items.get(res).map_or(false, |i| i.status == MarketStatus::Contraband)
        });

        if has_contraband {
            commands.entity(entity).insert(ContrabandPossession);
            // Trigger Notification/Crime Event
        }
    }
}
```

### 3. Integrate

Register systems in `Layer1SystemSet::Simulation`.
Add UI in Edict menu to toggle prohibitions.

## REFACTOR Phase: Quality & Design

- **Market Dynamics**: Contraband price should fluctuate based on enforcement level (Risk vs Reward).
- **Crime**: `ContrabandPossession` should decay if item is dropped/consumed.
- **Confiscation**: Militia should confiscate items from `ContrabandPossession` pops.

## Acceptance Criteria

- [ ] `Edict::Prohibition` exists.
- [ ] Banned items switch to `MarketStatus::Contraband`.
- [ ] Contraband items have 2x price (or similar markup).
- [ ] Pops holding contraband get marked with `ContrabandPossession`.
- [ ] Tests pass.

## Questions

*Builder: add questions here if spec is unclear.*
