# 382: Merchant Landings

## 1. Overview
**Layer:** Cross-layer (Layer 1 -> Layer 2)
**Fantasy:** The connection to the wider universe. The anticipation of new goods.
**Mechanic:** A "Trade Pad" allows merchant ships to land. They buy X and sell Y (randomized). Prices fluctuate.
**Emergence:** You survive a famine by selling your construction tech to a passing freighter for nutrient paste.
**Tension:** Sell strategic assets for immediate survival?

## 2. Dependencies
- Resource System (`ResourceType`, `ColonyResources`).
- Trading logic or inventory system.
- Trade Pad Building.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use bevy::prelude::*;

    #[test]
    fn test_merchant_ship_arrival() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, spawn_merchant_ship_system);

        // Add a trade pad
        app.world_mut().spawn(TradePad { active: true });

        // Act
        app.update();

        // Assert
        let merchant_ships = app.world().query::<&MerchantShip>().iter(app.world()).count();
        assert_eq!(merchant_ships, 1);
    }

    #[test]
    fn test_trade_transaction() {
        // Arrange
        let mut app = App::new();
        app.add_event::<TradeEvent>();
        app.add_systems(Update, process_trade_system);

        let mut resources = ColonyResources::default();
        resources.add_metal(100.0);
        app.insert_resource(resources);

        // Merchant selling Food for Metal
        let merchant_id = app.world_mut().spawn(MerchantShip {
            buying: ResourceType::Metal,
            selling: ResourceType::Food,
            exchange_rate: 2.0, // 2 Metal for 1 Food
            inventory: 50.0,
        }).id();

        app.world_mut().send_event(TradeEvent {
            merchant: merchant_id,
            buy_amount: 10.0, // Buying 10 Food
        });

        // Act
        app.update();

        // Assert
        let res = app.world().resource::<ColonyResources>();
        // Consumed 20 Metal for 10 Food
        assert_eq!(res.metal, 80.0);
        assert_eq!(res.food, 10.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::resources::{ColonyResources, ResourceType};

#[derive(Component)]
pub struct TradePad {
    pub active: bool,
}

#[derive(Component)]
pub struct MerchantShip {
    pub buying: ResourceType,
    pub selling: ResourceType,
    pub exchange_rate: f32,
    pub inventory: f32,
}

#[derive(Event)]
pub struct TradeEvent {
    pub merchant: Entity,
    pub buy_amount: f32,
}

pub fn spawn_merchant_ship_system(
    mut commands: Commands,
    pads: Query<&TradePad>,
    // In a real system, you'd use a timer to throttle arrivals
    ships: Query<&MerchantShip>,
) {
    if ships.iter().count() == 0 && pads.iter().any(|pad| pad.active) {
        commands.spawn(MerchantShip {
            buying: ResourceType::Metal, // Example hardcode
            selling: ResourceType::Food,
            exchange_rate: 2.0,
            inventory: 100.0,
        });
    }
}

pub fn process_trade_system(
    mut events: EventReader<TradeEvent>,
    mut merchants: Query<&mut MerchantShip>,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        if let Ok(mut ship) = merchants.get_mut(event.merchant) {
            let cost = event.buy_amount * ship.exchange_rate;

            // Simplified logic: assume consume returns true if successful
            if ship.inventory >= event.buy_amount && resources.consume(ship.buying, cost) {
                ship.inventory -= event.buy_amount;

                // Hardcoded addition for minimal implementation based on common resource setup
                match ship.selling {
                    ResourceType::Food => resources.add_food(event.buy_amount),
                    ResourceType::Metal => resources.add_metal(event.buy_amount),
                    _ => {} // Handle others
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Extract resource addition into a trait or helper method since `ColonyResources` doesn't have a generic `add_amount`.
- **Code Smells:** Hardcoded arrival condition. Needs a stochastic timer `Timer::from_seconds(X, TimerMode::Repeating)` combined with randomness.
- **Performance:** Trade processing is cheap and event-driven.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Merchant ships land if Trade Pads are present.
- [ ] Trades correctly deduct and add appropriate resources based on exchange rates.

## 7. Technical Guidance
- Integrate into the overarching simulation cycle (`SimulationTime`) for ship arrival and departure mechanics.
- A UI component will eventually need to read the `MerchantShip` components to display the trade window.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
