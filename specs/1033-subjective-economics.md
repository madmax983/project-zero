# 1033: Subjective Economics

## 1. Overview
Alien factions possess distinct "Value Matrices." What one species considers valuable, another considers waste (e.g., Lithoids paying a premium for Granite but treating Food as Bio-Waste). This allows players to exploit trade by acting as a middleman, such as becoming a "Garbage Dump" for a high-tech robotic civilization, importing their "Waste" (Alloys) and selling them your "Waste" (Rocks).

## 2. Dependencies
- Layer 3 `Diplomacy` (Factions, Species types).
- Layer 2/3 `Trade` economy and resource types.
- Layer 1 `Inventory` or `Cargo` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer3::trade::{MarketValue, TradeEvent, ResourceType};
    use crate::layer3::diplomacy::{Faction, SpeciesType};
    use crate::layer1::economy::Credits;

    #[test]
    fn test_resource_value_depends_on_faction_species() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_market_value_system);

        let lithoid_faction = app.world_mut().spawn((
            Faction { name: "Rock Eaters".to_string() },
            SpeciesType::Lithoid,
            MarketValue { resource: ResourceType::Granite, price: 0 },
        )).id();

        let human_faction = app.world_mut().spawn((
            Faction { name: "Humans".to_string() },
            SpeciesType::Human,
            MarketValue { resource: ResourceType::Granite, price: 0 },
        )).id();

        app.update();

        let lithoid_price = app.world().get::<MarketValue>(lithoid_faction).unwrap().price;
        let human_price = app.world().get::<MarketValue>(human_faction).unwrap().price;

        assert!(lithoid_price > human_price, "Lithoids should value Granite much higher than Humans.");
    }

    #[test]
    fn test_trading_trash_for_treasure() {
        let mut app = App::new();
        app.insert_resource(Credits { amount: 100 });
        app.add_event::<TradeEvent>();
        app.add_systems(Update, process_trade_system);

        // Setup Robot Faction selling "Scrap" (Alloys) cheaply
        let robot_faction = app.world_mut().spawn((
            Faction { name: "Robots".to_string() },
            SpeciesType::Robot,
            MarketValue { resource: ResourceType::Alloys, price: 1 }, // Dirt cheap for them
        )).id();

        app.world_mut().resource_mut::<Events<TradeEvent>>().send(TradeEvent {
            buyer: Entity::PLACEHOLDER, // Player
            seller: robot_faction,
            resource: ResourceType::Alloys,
            quantity: 10,
        });

        app.update();

        let credits = app.world().resource::<Credits>();
        assert_eq!(credits.amount, 90, "Player should only pay the subjective faction value for the resource.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer3/subjective_economics.rs
use bevy::prelude::*;
use crate::layer3::trade::{MarketValue, TradeEvent, ResourceType};
use crate::layer3::diplomacy::{Faction, SpeciesType};
use crate::layer1::economy::Credits;

pub fn evaluate_market_value_system(
    mut query: Query<(&SpeciesType, &mut MarketValue)>,
) {
    for (species, mut market) in query.iter_mut() {
        match species {
            SpeciesType::Lithoid => {
                if market.resource == ResourceType::Granite {
                    market.price = 50; // Premium
                } else if market.resource == ResourceType::Food {
                    market.price = 1; // Waste
                }
            },
            SpeciesType::Human => {
                if market.resource == ResourceType::Granite {
                    market.price = 5; // Base
                } else if market.resource == ResourceType::Food {
                    market.price = 10; // Base
                }
            },
            SpeciesType::Robot => {
                if market.resource == ResourceType::Alloys {
                    market.price = 1; // They generate so much it's waste
                }
            }
        }
    }
}

pub fn process_trade_system(
    mut events: EventReader<TradeEvent>,
    mut credits: ResMut<Credits>,
    market_query: Query<&MarketValue>,
) {
    for event in events.read() {
        if let Ok(market) = market_query.get(event.seller) {
            if market.resource == event.resource {
                let total_cost = market.price * event.quantity;
                if credits.amount >= total_cost {
                    credits.amount -= total_cost;
                    // In a full implementation, you'd add the resource to the player's inventory here
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Value Matrix Data:** Hardcoding the match statement is a bad idea. We need a `ResourceMatrix` asset file (JSON/RON) that defines base values and species multipliers.
- **Player Perspective:** The UI needs to show both the "Galactic Standard Value" and the "Faction Subjective Value" so the player understands the deal they are getting.
- **Trade Exhaustion:** Buying all of a robot's "trash" alloys shouldn't remain at price 1 forever. It should simulate supply/demand; eventually, they realize someone is buying it and raise the price.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_resource_value_depends_on_faction_species` passes.
- [ ] Test `test_trading_trash_for_treasure` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `Entity::PLACEHOLDER` is used in the test for the player, but you may need to use a dedicated Player Entity or Resource to prevent Bevy panics in the actual system depending on how your inventory is structured.

## 8. Questions
*Builder: add questions here if spec is unclear.*
