#![allow(clippy::type_complexity)]
//! Flower Bed Inspiration (Nova Feature).
//!
//! # The Spark
//! We have the `Trait::Artistic` personality trait and the aesthetic `BuildingType::FlowerBed`. We also have `ColonyResources::knowledge`. What if artists drew inspiration from nature?
//!
//! # The Feature
//! Pops with the `Trait::Artistic` trait who stand near a `FlowerBed` (within 2 tiles) passively regenerate their `leisure` (drawing inspiration from the beauty) and slowly generate `knowledge` for the colony each tick. However, this hyper-focus drains their `rest` slightly.
//!
//! # The Potential
//! This connects an aesthetic/vanity building (FlowerBed) directly to the colony economy (Knowledge) for specialized pops. It rewards players for creating dedicated gardens for their artists, turning beauty into a functional resource engine.

use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::core::map::GridPosition;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const INSPIRATION_RADIUS: u32 = 2;
const LEISURE_REGEN_PER_TICK: f32 = 0.02;
const KNOWLEDGE_GEN_PER_TICK: f32 = 0.05;
const REST_DRAIN_PER_TICK: f32 = 0.01;

pub fn flower_bed_inspiration_system(
    mut pops: Query<(&GridPosition, &Traits, &mut Needs), With<Pop>>,
    buildings: Query<(&GridPosition, &Building)>,
    mut resources: ResMut<ColonyResources>,
) {
    for (pop_pos, traits, mut needs) in pops.iter_mut() {
        if traits.has(Trait::Artistic) {
            let mut near_flower = false;
            for (b_pos, building) in buildings.iter() {
                if building.building_type == BuildingType::FlowerBed
                    && pop_pos.distance_chebyshev(*b_pos) <= INSPIRATION_RADIUS
                {
                    near_flower = true;
                    break;
                }
            }

            if near_flower {
                needs.leisure = (needs.leisure + LEISURE_REGEN_PER_TICK).min(1.0);
                needs.rest = (needs.rest - REST_DRAIN_PER_TICK).max(0.0);
                resources.add_knowledge(KNOWLEDGE_GEN_PER_TICK);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(flower_bed_inspiration_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_flower_bed_inspiration_system() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            knowledge: 10.0,
            max_knowledge: 100.0,
            ..Default::default()
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Artistic);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 5, y: 5 },
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Spawn FlowerBed nearby
        world.spawn((
            Building {
                building_type: BuildingType::FlowerBed,
            },
            GridPosition { x: 5, y: 6 }, // Distance 1
        ));

        world
            .run_system_once(flower_bed_inspiration_system)
            .unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Artistic pop should regenerate leisure"
        );
        assert!(needs.rest < 0.5, "Artistic pop should lose rest");

        let resources = world.get_resource::<ColonyResources>().unwrap();
        assert!(resources.knowledge > 10.0, "Should generate knowledge");
    }

    #[test]
    fn test_flower_bed_inspiration_too_far() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            knowledge: 10.0,
            max_knowledge: 100.0,
            ..Default::default()
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Artistic);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 5, y: 5 },
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Spawn FlowerBed too far
        world.spawn((
            Building {
                building_type: BuildingType::FlowerBed,
            },
            GridPosition { x: 5, y: 8 }, // Distance 3
        ));

        world
            .run_system_once(flower_bed_inspiration_system)
            .unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert_eq!(needs.leisure, 0.5, "Should not regenerate leisure");
        assert_eq!(needs.rest, 0.5, "Should not lose rest");

        let resources = world.get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.knowledge, 10.0, "Should not generate knowledge");
    }

    #[test]
    fn test_flower_bed_non_artistic() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            knowledge: 10.0,
            max_knowledge: 100.0,
            ..Default::default()
        });

        let pop = world
            .spawn((
                Pop,
                Traits::default(), // Not artistic
                GridPosition { x: 5, y: 5 },
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            Building {
                building_type: BuildingType::FlowerBed,
            },
            GridPosition { x: 5, y: 6 },
        ));

        world
            .run_system_once(flower_bed_inspiration_system)
            .unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert_eq!(needs.leisure, 0.5, "Normal pop should not get inspiration");

        let resources = world.get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.knowledge, 10.0, "Should not generate knowledge");
    }
}
