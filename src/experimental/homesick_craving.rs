#![allow(clippy::type_complexity)]
//! Homesick Craving (Nova Feature).
//!
//! # The Spark
//! We have the `Trait::Homesick` and `Trait::Xenophobic` personality traits, and physiological/psychological `Needs` (`leisure`). We also have a `BuildingType::TradeDepot`.
//!
//! # The Feature
//! Pops with the `Trait::Homesick` or `Trait::Xenophobic` traits feel isolated on the new colony and long for familiar things.
//! When they stand near a `TradeDepot` (within 3 tiles), which represents a connection to their home planet or their own kind (through imported goods or merchants), they passively regenerate their `leisure` need, feeling a sense of comfort and connection.
//!
//! # The Potential
//! This connects a personality trait with a specific economic building's physical location. It makes an otherwise industrial/trade building an emergent leisure structure for specific populations.
//! Players might intentionally build Trade Depots near residential zones housing these specific pops to manage their stress, despite the potential downsides of having merchants nearby.

use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::core::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const TRADE_DEPOT_RADIUS: u32 = 3;
const HOMESICK_LEISURE_REGEN: f32 = 0.02;

pub fn homesick_craving_system(
    mut pops: Query<(&GridPosition, &Traits, &mut Needs), With<Pop>>,
    buildings: Query<(&GridPosition, &Building)>,
) {
    for (pop_pos, traits, mut needs) in pops.iter_mut() {
        if traits.has(Trait::Homesick) || traits.has(Trait::Xenophobic) {
            let mut near_depot = false;
            for (b_pos, building) in buildings.iter() {
                if building.building_type == BuildingType::TradeDepot
                    && pop_pos.distance_chebyshev(*b_pos) <= TRADE_DEPOT_RADIUS
                {
                    near_depot = true;
                    break;
                }
            }

            if near_depot {
                needs.leisure = (needs.leisure + HOMESICK_LEISURE_REGEN).min(1.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(homesick_craving_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_homesick_craving_effects() {
        let mut world = World::new();

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Homesick);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 5, y: 5 },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Spawn Trade Depot nearby
        world.spawn((
            Building {
                building_type: BuildingType::TradeDepot,
            },
            GridPosition { x: 5, y: 7 }, // Distance 2
        ));

        world.run_system_once(homesick_craving_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Homesick pop should regenerate leisure when near Trade Depot"
        );
    }

    #[test]
    fn test_homesick_far_away() {
        let mut world = World::new();

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Homesick);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Spawn Trade Depot far away
        world.spawn((
            Building {
                building_type: BuildingType::TradeDepot,
            },
            GridPosition { x: 10, y: 10 }, // Distance > 3
        ));

        world.run_system_once(homesick_craving_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Homesick pop should not regenerate leisure when far from Trade Depot"
        );
    }

    #[test]
    fn test_xenophobic_near_depot() {
        let mut world = World::new();

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Xenophobic);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 5, y: 5 },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::TradeDepot,
            },
            GridPosition { x: 5, y: 6 },
        ));

        world.run_system_once(homesick_craving_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Xenophobic pop should regenerate leisure when near Trade Depot"
        );
    }

    #[test]
    fn test_non_homesick_near_depot() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                Traits::default(),
                GridPosition { x: 5, y: 5 },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::TradeDepot,
            },
            GridPosition { x: 5, y: 6 },
        ));

        world.run_system_once(homesick_craving_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Normal pop should not regenerate leisure from Trade Depot"
        );
    }
}
