#![allow(clippy::type_complexity)]
//! Rainy Day Reading (Nova Feature).
//!
//! # The Spark
//! We have the `Trait::Intellectual`, `WeatherType::Rain`, and `BuildingType::Library`. Do intellectuals enjoy reading when it rains?
//!
//! # The Feature
//! Pops with the `Intellectual` trait who are sheltered under a roof near a `Library` during `Rain` or `Storm` weather
//! will rapidly regenerate their `leisure` need and passively generate a small amount of `knowledge` for the colony.
//!
//! # The Potential
//! Creates a niche interaction where bad weather is actually highly beneficial for a specific colony demographic.

use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::needs::Needs;
use crate::layer1::physics::structural_integrity::RoofGrid;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

const READING_RADIUS: u32 = 5;
const LEISURE_REGEN_PER_TICK: f32 = 0.05;
const KNOWLEDGE_GEN_PER_TICK: f32 = 0.02;

pub fn rainy_day_reading_system(
    mut pops: Query<(&GridPosition, &Traits, &mut Needs), With<Pop>>,
    libraries: Query<(&GridPosition, &Building)>,
    weather: Option<Res<WeatherState>>,
    roof_grid: Option<Res<RoofGrid>>,
    mut resources: Option<ResMut<ColonyResources>>,
) {
    let is_raining = if let Some(w) = weather {
        w.current_weather == WeatherType::Rain || w.current_weather == WeatherType::Storm
    } else {
        false
    };

    if !is_raining {
        return;
    }

    let mut library_positions = Vec::new();
    for (pos, building) in libraries.iter() {
        if building.building_type == BuildingType::Library {
            library_positions.push(*pos);
        }
    }

    if library_positions.is_empty() {
        return;
    }

    let has_roof_grid = roof_grid.is_some();
    let roofs = roof_grid.as_deref();

    let mut generated_knowledge = 0.0;

    for (pos, traits, mut needs) in &mut pops {
        if traits.has(Trait::Intellectual) {
            let mut under_roof = true;
            if has_roof_grid {
                if let Some(r) = roofs {
                    under_roof = r.has_roof(pos.x, pos.y);
                }
            }

            if under_roof {
                let mut near_library = false;
                for lib_pos in &library_positions {
                    if pos.distance_chebyshev(*lib_pos) <= READING_RADIUS {
                        near_library = true;
                        break;
                    }
                }

                if near_library {
                    needs.leisure = (needs.leisure + LEISURE_REGEN_PER_TICK).min(1.0);
                    generated_knowledge += KNOWLEDGE_GEN_PER_TICK;
                }
            }
        }
    }

    if generated_knowledge > 0.0 {
        if let Some(res) = resources.as_mut() {
            res.add_knowledge(generated_knowledge);
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(rainy_day_reading_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_rainy_day_reading() {
        let mut world = World::new();

        let res = ColonyResources {
            max_knowledge: 100.0,
            ..Default::default()
        };
        world.insert_resource(res);
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Rain,
            ..Default::default()
        });

        let mut roof_grid = RoofGrid::new(10, 10);
        // RoofGrid doesn't have `add_roof`, we use `set` to set roof true at (5,5)
        roof_grid.set(5, 5, true);
        world.insert_resource(roof_grid);

        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Library,
            },
        ));

        let mut traits = Traits::default();
        traits.add(Trait::Intellectual);

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            traits,
            Needs {
                leisure: 0.5,
                ..Default::default()
            },
        ));

        world.run_system_once(rainy_day_reading_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge > 0.0);
    }
}
