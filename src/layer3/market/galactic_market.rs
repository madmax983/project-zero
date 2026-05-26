//! Galactic Market Core
//!
//! The central hub of interstellar trade, driving ruthless price fluctuations.
//! Supply and demand here dictate the wealth of empires.

use crate::layer1::resources::ResourceType;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// The Galactic Market tracking global prices and supply pools.
#[derive(Resource)]
pub struct GalacticMarket {
    pub prices: HashMap<ResourceType, f32>,
    pub supply_pool: HashMap<ResourceType, f32>,
    pub baseline_prices: HashMap<ResourceType, f32>,
}

impl Default for GalacticMarket {
    fn default() -> Self {
        Self::new()
    }
}

impl GalacticMarket {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        let mut supply_pool = HashMap::new();
        let mut baseline_prices = HashMap::new();
        // Initialize with default values.
        let default_resources = vec![
            ResourceType::Food,
            ResourceType::Wood,
            ResourceType::Stone,
            ResourceType::Ore,
            ResourceType::Metal,
            ResourceType::Planks,
            ResourceType::Blocks,
            ResourceType::Waste,
            ResourceType::Rations,
            ResourceType::Fuel,
            ResourceType::Alcohol,
            ResourceType::Scrap,
            ResourceType::Tools,
            ResourceType::BuildingPermit,
            ResourceType::MemoryCore,
        ];

        for r in default_resources {
            prices.insert(r, 10.0);
            supply_pool.insert(r, 1000.0);
            baseline_prices.insert(r, 10.0);
        }

        Self {
            prices,
            supply_pool,
            baseline_prices,
        }
    }
}

/// Executes a sell order on the galactic market, increasing the supply pool.
///
/// If the amount is less than or equal to 0, or is not finite, the function will return early
/// without modifying the market state.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::resources::ResourceType;
/// use scale::layer3::market::galactic_market::{GalacticMarket, execute_market_sell};
///
/// let mut world = World::new();
/// world.insert_resource(GalacticMarket::new());
///
/// // Execute a sell order for 100 units of Food
/// execute_market_sell(&mut world, ResourceType::Food, 100.0);
///
/// let market = world.resource::<GalacticMarket>();
/// // Original pool is 1000.0 + 100.0 = 1100.0
/// assert_eq!(market.supply_pool.get(&ResourceType::Food).copied(), Some(1100.0));
/// ```
pub fn execute_market_sell(world: &mut World, resource: ResourceType, amount: f32) {
    if amount <= 0.0 || !amount.is_finite() {
        return;
    }
    let mut market = world.resource_mut::<GalacticMarket>();
    let pool = market.supply_pool.entry(resource).or_insert(0.0);
    *pool += amount;
}

/// Executes a buy order on the galactic market, decreasing the supply pool.
///
/// If the amount is less than or equal to 0, or is not finite, the function will return early
/// without modifying the market state. The supply pool will never drop below `0.0`.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::resources::ResourceType;
/// use scale::layer3::market::galactic_market::{GalacticMarket, execute_market_buy};
///
/// let mut world = World::new();
/// world.insert_resource(GalacticMarket::new());
///
/// // Execute a buy order for 50 units of Ore
/// execute_market_buy(&mut world, ResourceType::Ore, 50.0);
///
/// let market = world.resource::<GalacticMarket>();
/// // Original pool is 1000.0 - 50.0 = 950.0
/// assert_eq!(market.supply_pool.get(&ResourceType::Ore).copied(), Some(950.0));
/// ```
pub fn execute_market_buy(world: &mut World, resource: ResourceType, amount: f32) {
    if amount <= 0.0 || !amount.is_finite() {
        return;
    }
    let mut market = world.resource_mut::<GalacticMarket>();
    let pool = market.supply_pool.entry(resource).or_insert(0.0);
    *pool = (*pool - amount).max(0.0);
}

pub fn calculate_trade_value(world: &World, resource: ResourceType, amount: f32) -> f32 {
    let market = world.resource::<GalacticMarket>();
    let price = market.prices.get(&resource).copied().unwrap_or(0.0);
    price * amount
}

pub fn update_market_prices_system(mut market: ResMut<GalacticMarket>) {
    let mut new_prices = HashMap::new();
    for (res, pool) in &market.supply_pool {
        let safe_pool = pool.max(1.0);
        let baseline = market.baseline_prices.get(res).copied().unwrap_or(10.0);

        let target_price = baseline * (1000.0 / safe_pool);

        // REFACTOR Phase: Price floor and ceiling
        let floor = baseline * 0.1; // Price cannot drop below 10% of baseline
        let ceiling = baseline * 10.0; // Price cannot exceed 1000% of baseline
        let target_price = target_price.clamp(floor, ceiling);

        // REFACTOR Phase: Slow "normalization" mechanic.
        let current_price = market.prices.get(res).copied().unwrap_or(baseline);

        // 5% move towards the target price, and a very slow 1% drift to baseline
        let new_price = current_price + (target_price - current_price) * 0.05;
        let final_price = new_price + (baseline - new_price) * 0.01;

        new_prices.insert(*res, final_price);
    }
    for (res, price) in new_prices {
        market.prices.insert(res, price);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_galactic_market_new() {
        let market = GalacticMarket::new();
        assert_eq!(market.prices.len(), 15);
        assert_eq!(market.prices.get(&ResourceType::Food).copied(), Some(10.0));
        assert_eq!(
            market.supply_pool.get(&ResourceType::Wood).copied(),
            Some(1000.0)
        );
    }

    #[test]
    fn test_execute_market_sell_invalid() {
        let mut world = World::new();
        world.insert_resource(GalacticMarket::new());
        execute_market_sell(&mut world, ResourceType::Food, -10.0);
        execute_market_sell(&mut world, ResourceType::Food, f32::INFINITY);

        let market = world.resource::<GalacticMarket>();
        assert_eq!(
            market.supply_pool.get(&ResourceType::Food).copied(),
            Some(1000.0)
        );
    }

    #[test]
    fn test_execute_market_buy_invalid() {
        let mut world = World::new();
        world.insert_resource(GalacticMarket::new());
        execute_market_buy(&mut world, ResourceType::Food, -10.0);
        execute_market_buy(&mut world, ResourceType::Food, f32::INFINITY);

        let market = world.resource::<GalacticMarket>();
        assert_eq!(
            market.supply_pool.get(&ResourceType::Food).copied(),
            Some(1000.0)
        );
    }

    #[test]
    fn test_update_market_prices_system_missing_baseline() {
        let mut world = World::new();
        world.insert_resource(GalacticMarket {
            prices: vec![(ResourceType::Food, 10.0)].into_iter().collect(),
            supply_pool: vec![(ResourceType::Food, 1000.0)].into_iter().collect(),
            baseline_prices: vec![].into_iter().collect(), // Missing baseline for Food
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_market_prices_system);
        schedule.run(&mut world);

        let market = world.resource::<GalacticMarket>();
        // Missing baseline should default to 10.0, so price stays at 10.0
        assert_eq!(market.prices.get(&ResourceType::Food).copied(), Some(10.0));
    }

    #[test]
    fn test_galactic_market_price_fluctuation() {
        let mut world = World::new();
        // Arrange: Initialize GalacticMarket resource with base prices.
        world.insert_resource(GalacticMarket {
            prices: vec![(ResourceType::Food, 10.0)].into_iter().collect(),
            supply_pool: vec![(ResourceType::Food, 1000.0)].into_iter().collect(),
            baseline_prices: vec![(ResourceType::Food, 10.0)].into_iter().collect(),
        });

        // Act: Simulate a massive sell order for a specific resource (e.g., Food).
        execute_market_sell(&mut world, ResourceType::Food, 5000.0);

        // Use a system scheduler so we can run `update_market_prices_system`.
        let mut schedule = Schedule::default();
        schedule.add_systems(update_market_prices_system);
        schedule.run(&mut world);

        // Assert: The market price for Food decreases proportionally.
        let market = world.resource::<GalacticMarket>();
        let new_price = market.prices.get(&ResourceType::Food).unwrap();
        assert!(
            *new_price < 10.0,
            "Massive sell order should crash the price of Food."
        );
    }

    #[test]
    fn test_market_buy_increases_price() {
        let mut world = World::new();
        // Arrange: Initialize GalacticMarket resource.
        world.insert_resource(GalacticMarket {
            prices: vec![(ResourceType::Metal, 50.0)].into_iter().collect(),
            supply_pool: vec![(ResourceType::Metal, 100.0)].into_iter().collect(),
            baseline_prices: vec![(ResourceType::Metal, 50.0)].into_iter().collect(),
        });

        // Act: Simulate a massive buy order for a resource.
        execute_market_buy(&mut world, ResourceType::Metal, 50.0); // Now pool is 50.0

        let mut schedule = Schedule::default();
        schedule.add_systems(update_market_prices_system);
        schedule.run(&mut world);

        // Assert: The market price for that resource increases.
        let market = world.resource::<GalacticMarket>();
        let new_price = market.prices.get(&ResourceType::Metal).unwrap();
        // pool was 100, now 50.
        // With simple math: 10.0 * (1000 / 50) = 200.0
        // We initially had 50.0 price, so we need to test if the new_price is > 50.0
        assert!(
            *new_price > 50.0,
            "Massive buy order should spike the price of Metal."
        );
    }

    #[test]
    fn test_trade_execution_uses_current_market_price() {
        let mut world = World::new();
        // Arrange: Set a specific market price for a resource.
        world.insert_resource(GalacticMarket {
            prices: vec![(ResourceType::Fuel, 25.0)].into_iter().collect(),
            supply_pool: vec![].into_iter().collect(),
            baseline_prices: vec![(ResourceType::Fuel, 25.0)].into_iter().collect(),
        });

        // Act: Execute a trade order.
        let credits_earned = calculate_trade_value(&world, ResourceType::Fuel, 10.0);

        // Assert: The credits exchanged exactly match the quantity * market price.
        assert_eq!(
            credits_earned, 250.0,
            "Trade value should be quantity * dynamic market price."
        );
    }
}
