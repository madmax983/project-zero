//! Martial Vigilance (Nova Feature).
//!
//! # The Spark
//! We have the `Trait::Militaristic` personality trait, defensive structures like `BuildingType::Tower` and `BuildingType::Gate`, and physiological `Needs`.
//!
//! # The Feature
//! Pops with the `Trait::Militaristic` find deep psychological comfort in fortified areas.
//! When standing within a 3-tile radius (Chebyshev distance) of a `Tower` or `Gate`, they
//! enter a state of "Martial Vigilance". This passively regenerates their `leisure` need, as
//! standing guard gives them a sense of purpose. However, the hyper-focus causes their
//! `rest` need to decay slightly faster.
//!
//! # The Potential
//! This creates an emergent base design strategy where players can station their militaristic
//! pops near the colony's borders or guard posts to keep them entertained and fulfilled,
//! converting defensive perimeters into psychological havens.

use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::core::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const VIGILANCE_RADIUS: u32 = 3;
const LEISURE_REGEN: f32 = 0.05;
const REST_DECAY: f32 = 0.02;

pub fn martial_vigilance_system(
    mut pops: Query<(&GridPosition, &mut Needs, &Traits), With<Pop>>,
    buildings: Query<(&GridPosition, &Building)>,
) {
    for (pop_pos, mut needs, traits) in pops.iter_mut() {
        if traits.has(Trait::Militaristic) {
            let mut is_near_defense = false;
            for (building_pos, building) in buildings.iter() {
                if (building.building_type == BuildingType::Tower
                    || building.building_type == BuildingType::Gate)
                    && pop_pos.distance_chebyshev(*building_pos) <= VIGILANCE_RADIUS
                {
                    is_near_defense = true;
                    break;
                }
            }

            if is_near_defense {
                // Passively regenerate leisure, but decay rest
                needs.leisure = (needs.leisure + LEISURE_REGEN).min(1.0);
                needs.rest = (needs.rest - REST_DECAY).max(0.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(martial_vigilance_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_militaristic_near_tower() {
        let mut world = World::new();

        // Spawn Tower
        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Tower,
            },
        ));

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Militaristic);
            t
        };

        // Spawn Militaristic Pop nearby
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 6 }, // Within radius 3
                traits,
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(martial_vigilance_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Militaristic should regenerate leisure near Tower"
        );
        assert!(needs.rest < 0.5, "Militaristic should lose rest near Tower");
    }

    #[test]
    fn test_militaristic_far_from_tower() {
        let mut world = World::new();

        // Spawn Tower
        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Tower,
            },
        ));

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Militaristic);
            t
        };

        // Spawn Militaristic Pop far away
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 20, y: 20 }, // Outside radius 3
                traits,
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(martial_vigilance_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Militaristic should not regenerate leisure when far from Tower"
        );
        assert!(
            (needs.rest - 0.5).abs() < f32::EPSILON,
            "Militaristic should not lose rest when far from Tower"
        );
    }

    #[test]
    fn test_non_militaristic_near_tower() {
        let mut world = World::new();

        // Spawn Tower
        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Tower,
            },
        ));

        // Spawn non-Militaristic Pop nearby
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 6 }, // Within radius 3
                Traits::default(),           // Not Militaristic
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(martial_vigilance_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Non-Militaristic should not regenerate leisure near Tower"
        );
        assert!(
            (needs.rest - 0.5).abs() < f32::EPSILON,
            "Non-Militaristic should not lose rest near Tower"
        );
    }
}
