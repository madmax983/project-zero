//! Panic Buying (Nova Feature).
//!
//! # The Spark
//! We have a `Morale` system and `ColonyPrices` in the economy. What happens when
//! the colony's collective morale tanks?
//!
//! # The Feature
//! The `panic_buying_system` monitors the average morale of all Pops. If the average
//! falls below a critical threshold (e.g., 0.3), the colony enters a state of panic
//! buying. This causes the `ColonyPrices` for food and luxuries to skyrocket, reflecting
//! scarcity fears and hoarding behavior. Prices normalize when morale recovers.

use crate::layer1::economy::ColonyPrices;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

const PANIC_THRESHOLD: f32 = 0.3;
const PANIC_MULTIPLIER: f32 = 3.0;
const NORMAL_FOOD_PRICE: f32 = 1.0;
const NORMAL_LUXURY_PRICE: f32 = 5.0;

/// System that adjusts ColonyPrices based on the average Morale of the colony.
pub fn panic_buying_system(pops: Query<&Morale, With<Pop>>, prices: Option<ResMut<ColonyPrices>>) {
    if pops.is_empty() {
        return;
    }

    if let Some(mut prices) = prices {
        let mut total_morale = 0.0;
        let mut count = 0;

        for morale in pops.iter() {
            total_morale += morale.value;
            count += 1;
        }

        let average_morale = total_morale / count as f32;

        if average_morale < PANIC_THRESHOLD {
            // Panic buying active! Prices skyrocket.
            prices.food_price = NORMAL_FOOD_PRICE * PANIC_MULTIPLIER;
            prices.luxury_price = NORMAL_LUXURY_PRICE * PANIC_MULTIPLIER;
        } else {
            // Morale is stable. Normal prices.
            prices.food_price = NORMAL_FOOD_PRICE;
            prices.luxury_price = NORMAL_LUXURY_PRICE;
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(panic_buying_system);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_buying_triggers_price_spike() {
        let mut world = World::new();
        world.insert_resource(ColonyPrices {
            food_price: NORMAL_FOOD_PRICE,
            luxury_price: NORMAL_LUXURY_PRICE,
        });

        // Spawn pops with low morale
        world.spawn((
            Pop,
            Morale {
                value: 0.1,
                modifiers: vec![],
            },
        ));
        world.spawn((
            Pop,
            Morale {
                value: 0.2,
                modifiers: vec![],
            },
        ));

        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, panic_buying_system);

        let prices = world.resource::<ColonyPrices>();
        assert_eq!(prices.food_price, NORMAL_FOOD_PRICE * PANIC_MULTIPLIER);
        assert_eq!(prices.luxury_price, NORMAL_LUXURY_PRICE * PANIC_MULTIPLIER);
    }

    #[test]
    fn test_panic_buying_normal_prices() {
        let mut world = World::new();
        // Start with spiked prices to ensure they normalize
        world.insert_resource(ColonyPrices {
            food_price: NORMAL_FOOD_PRICE * PANIC_MULTIPLIER,
            luxury_price: NORMAL_LUXURY_PRICE * PANIC_MULTIPLIER,
        });

        // Spawn pops with high morale
        world.spawn((
            Pop,
            Morale {
                value: 0.8,
                modifiers: vec![],
            },
        ));
        world.spawn((
            Pop,
            Morale {
                value: 0.9,
                modifiers: vec![],
            },
        ));

        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, panic_buying_system);

        let prices = world.resource::<ColonyPrices>();
        assert_eq!(prices.food_price, NORMAL_FOOD_PRICE);
        assert_eq!(prices.luxury_price, NORMAL_LUXURY_PRICE);
    }
}
