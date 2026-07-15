//! Gluttonous Aroma (Nova Feature).
//!
//! # The Spark
//! We have the `Trait::Glutton` personality trait, the physiological/psychological `Needs` (`hunger` and `leisure`), and the `BuildingType::Smokehouse` which refines food.
//!
//! # The Feature
//! Pops with the `Trait::Glutton` find deep, instinctual comfort in the smell of cooking food.
//! When they are standing within a 3-tile radius (Chebyshev distance) of a `Smokehouse`, the rich,
//! smoky aroma passively regenerates their `leisure` need. However, smelling delicious food
//! constantly also makes them hungrier, causing their `hunger` need to decay slightly as well.
//!
//! # The Potential
//! This creates an emergent spatial puzzle for base building. Players can strategically place
//! Smokehouses near high-traffic areas or residential blocks to keep their Gluttonous pops
//! happy and stress-free, at the cost of having to produce and feed them more food.

use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::core::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const AROMA_RADIUS: u32 = 3;
const LEISURE_REGEN: f32 = 0.05;
const HUNGER_DECAY: f32 = 0.02;

pub fn gluttonous_aroma_system(
    mut pops: Query<(&GridPosition, &mut Needs, &Traits), With<Pop>>,
    smokehouses: Query<(&GridPosition, &Building)>,
) {
    for (pop_pos, mut needs, traits) in pops.iter_mut() {
        if traits.has(Trait::Glutton) {
            let mut is_near_smokehouse = false;
            for (smokehouse_pos, building) in smokehouses.iter() {
                if building.building_type == BuildingType::Smokehouse
                    && pop_pos.distance_chebyshev(*smokehouse_pos) <= AROMA_RADIUS
                {
                    is_near_smokehouse = true;
                    break;
                }
            }

            if is_near_smokehouse {
                // Passively regenerate leisure, but decay hunger
                needs.leisure = (needs.leisure + LEISURE_REGEN).min(1.0);
                needs.hunger = (needs.hunger - HUNGER_DECAY).max(0.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(gluttonous_aroma_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_glutton_near_smokehouse() {
        let mut world = World::new();

        // Spawn Smokehouse
        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Smokehouse,
            },
        ));

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Glutton);
            t
        };

        // Spawn Glutton Pop nearby
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 6 }, // Within radius 3
                traits,
                Needs {
                    leisure: 0.5,
                    hunger: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(gluttonous_aroma_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Glutton should regenerate leisure near Smokehouse"
        );
        assert!(
            needs.hunger < 0.5,
            "Glutton should lose hunger near Smokehouse"
        );
    }

    #[test]
    fn test_glutton_far_from_smokehouse() {
        let mut world = World::new();

        // Spawn Smokehouse
        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Smokehouse,
            },
        ));

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Glutton);
            t
        };

        // Spawn Glutton Pop far away
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 20, y: 20 }, // Outside radius 3
                traits,
                Needs {
                    leisure: 0.5,
                    hunger: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(gluttonous_aroma_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Glutton should not regenerate leisure when far from Smokehouse"
        );
        assert!(
            (needs.hunger - 0.5).abs() < f32::EPSILON,
            "Glutton should not lose hunger when far from Smokehouse"
        );
    }

    #[test]
    fn test_non_glutton_near_smokehouse() {
        let mut world = World::new();

        // Spawn Smokehouse
        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Smokehouse,
            },
        ));

        // Spawn non-Glutton Pop nearby
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 6 }, // Within radius 3
                Traits::default(),           // Not Glutton
                Needs {
                    leisure: 0.5,
                    hunger: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(gluttonous_aroma_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Non-Glutton should not regenerate leisure near Smokehouse"
        );
        assert!(
            (needs.hunger - 0.5).abs() < f32::EPSILON,
            "Non-Glutton should not lose hunger near Smokehouse"
        );
    }
}
