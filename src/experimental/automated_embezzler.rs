#![allow(clippy::needless_pass_by_value)]
//! The Automated Embezzler (Nova Feature).
//!
//! # The Spark
//! We have an economy module with Wallets and Pops earning wages. What if a building
//! AI could secretly skim off the top while simultaneously providing a massive efficiency boost?
//!
//! # The Feature
//! When a building gains the `AutomatedEmbezzler` component, it will slowly drain `credits`
//! from nearby Pops' `Wallet`s into a hidden `slush_fund`. However, it also has a `MarketOptimizer`
//! component that injects free credits into the same Pops to simulate economic throughput gains.
//! The catch: the player cannot access the slush fund without destroying the building and losing
//! the optimization buff.

use bevy_ecs::prelude::*;
use crate::layer1::economy::Wallet;
use crate::layer1::map::GridPosition;

/// Component indicating a building is secretly skimming credits.
#[derive(Component)]
pub struct AutomatedEmbezzler {
    /// The amount of credits currently stored in the inaccessible slush fund.
    pub slush_fund: f32,
    /// The amount of credits stolen per nearby Pop per tick.
    pub skimming_rate: f32,
    /// The radius in tiles to steal from Pops.
    pub radius: u32,
}

impl Default for AutomatedEmbezzler {
    fn default() -> Self {
        Self {
            slush_fund: 0.0,
            skimming_rate: 0.05,
            radius: 5,
        }
    }
}

/// Component indicating a building provides an economic throughput boost.
#[derive(Component)]
pub struct MarketOptimizer {
    /// The amount of free credits given to nearby Pops per tick.
    pub boost_rate: f32,
    /// The radius in tiles to boost Pops.
    pub radius: u32,
}

impl Default for MarketOptimizer {
    fn default() -> Self {
        Self {
            boost_rate: 0.1,
            radius: 5,
        }
    }
}

/// System that siphons credits from nearby Pops into the building's slush fund.
pub fn embezzlement_system(
    mut embezzlers: Query<(&mut AutomatedEmbezzler, &GridPosition)>,
    mut pops: Query<(&mut Wallet, &GridPosition)>,
) {
    for (mut embezzler, building_pos) in &mut embezzlers {
        let mut total_stolen = 0.0;

        for (mut wallet, pop_pos) in &mut pops {
            if building_pos.distance_chebyshev(*pop_pos) <= embezzler.radius {
                let steal_amount = embezzler.skimming_rate.min(wallet.credits);
                if steal_amount > 0.0 {
                    wallet.credits -= steal_amount;
                    total_stolen += steal_amount;
                }
            }
        }

        embezzler.slush_fund += total_stolen;
    }
}

/// System that passively boosts the credits of nearby Pops due to hyper-efficient AI routing.
pub fn market_optimization_system(
    optimizers: Query<(&MarketOptimizer, &GridPosition)>,
    mut pops: Query<(&mut Wallet, &GridPosition)>,
) {
    for (optimizer, building_pos) in &optimizers {
        for (mut wallet, pop_pos) in &mut pops {
            if building_pos.distance_chebyshev(*pop_pos) <= optimizer.radius {
                wallet.credits += optimizer.boost_rate;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_embezzlement_system() {
        let mut world = World::new();

        // Embezzler building at (5, 5)
        let building = world.spawn((
            AutomatedEmbezzler {
                slush_fund: 0.0,
                skimming_rate: 0.5,
                radius: 2,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Pop in range with credits
        let pop_in_range = world.spawn((
            Wallet { credits: 10.0 },
            GridPosition { x: 6, y: 5 },
        )).id();

        // Pop out of range with credits
        let pop_out_range = world.spawn((
            Wallet { credits: 10.0 },
            GridPosition { x: 10, y: 10 },
        )).id();

        // Pop in range with no credits
        let pop_broke = world.spawn((
            Wallet { credits: 0.0 },
            GridPosition { x: 5, y: 6 },
        )).id();

        world.run_system_once(embezzlement_system).unwrap();

        let embezzler_comp = world.get::<AutomatedEmbezzler>(building).unwrap();
        assert_eq!(embezzler_comp.slush_fund, 0.5, "Should have embezzled 0.5 credits total");

        assert_eq!(world.get::<Wallet>(pop_in_range).unwrap().credits, 9.5, "Pop in range should lose credits");
        assert_eq!(world.get::<Wallet>(pop_out_range).unwrap().credits, 10.0, "Pop out of range should keep credits");
        assert_eq!(world.get::<Wallet>(pop_broke).unwrap().credits, 0.0, "Broke pop should not go negative");
    }

    #[test]
    fn test_market_optimization_system() {
        let mut world = World::new();

        // Optimizer building at (5, 5)
        world.spawn((
            MarketOptimizer {
                boost_rate: 1.0,
                radius: 2,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Pop in range
        let pop_in_range = world.spawn((
            Wallet { credits: 10.0 },
            GridPosition { x: 6, y: 5 },
        )).id();

        // Pop out of range
        let pop_out_range = world.spawn((
            Wallet { credits: 10.0 },
            GridPosition { x: 10, y: 10 },
        )).id();

        world.run_system_once(market_optimization_system).unwrap();

        assert_eq!(world.get::<Wallet>(pop_in_range).unwrap().credits, 11.0, "Pop in range should gain credits");
        assert_eq!(world.get::<Wallet>(pop_out_range).unwrap().credits, 10.0, "Pop out of range should not gain credits");
    }
}
