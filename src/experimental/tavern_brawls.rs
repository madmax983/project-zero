#![allow(clippy::type_complexity)]
//! Tavern Brawls (Nova Feature).
//!
//! # The Spark
//! We have a `Tavern` building where Pops go to `Socialize`, and a `Morale` system.
//! What happens when miserable Pops gather to drink?
//!
//! # The Feature
//! If multiple Pops with very low `Morale` are socializing in the same `Tavern`, a `TavernBrawl` can erupt.
//! This creates a physical mess (spawns `Clutter`) and damages the participants (`Health`), turning
//! a place of relaxation into a hazard during times of colony-wide depression.
//!
//! # The Potential
//! Connects the social infrastructure (`Tavern`) directly to the psychological (`Morale`) and
//! physical (`Health`, `ClutterGrid`) systems. Players must balance keeping their Pops happy
//! with the risk of mass gatherings turning violent when morale inevitably dips.

use crate::layer1::biology::health::Health;
use crate::layer1::clutter::ClutterGrid;
use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use crate::layer1::social::Tavern;
use bevy_ecs::prelude::*;
use rand::Rng;

const BRAWL_MORALE_THRESHOLD: f32 = 0.3; // Pops below 30% morale are prone to brawling
const BRAWL_CHANCE: f32 = 0.05; // 5% chance per tick if conditions are met
const BRAWL_DAMAGE: f32 = 10.0;
const BRAWL_CLUTTER: f32 = 20.0;

/// System that triggers and handles Tavern Brawls.
pub fn tavern_brawls_system(
    mut taverns: Query<(&Tavern, &GridPosition)>,
    mut pops: Query<(&PopAction, &Morale, &mut Health), With<Pop>>,
    mut clutter_grid: Option<ResMut<ClutterGrid>>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    let mut rng = rand::thread_rng();

    for (tavern, tavern_pos) in taverns.iter_mut() {
        // We need at least 2 visitors for a brawl
        if tavern.visitors.len() < 2 {
            continue;
        }

        let mut miserable_count = 0;
        let mut brawlers = Vec::new();

        for &visitor_entity in &tavern.visitors {
            if let Ok((action, morale, _)) = pops.get(visitor_entity) {
                if action.current == ActionType::Socialize && morale.value < BRAWL_MORALE_THRESHOLD
                {
                    miserable_count += 1;
                    brawlers.push(visitor_entity);
                }
            }
        }

        // If multiple miserable pops are socializing, a brawl might start
        if miserable_count >= 2 && rng.gen::<f32>() < BRAWL_CHANCE {
            // A brawl occurs!

            // Damage the brawlers
            for &brawler_entity in &brawlers {
                if let Ok((_, _, mut health)) = pops.get_mut(brawler_entity) {
                    health.current = (health.current - BRAWL_DAMAGE).max(0.0);
                }
            }

            // Create a mess
            if let Some(ref mut grid) = clutter_grid {
                if let (Ok(x), Ok(y)) =
                    (usize::try_from(tavern_pos.x), usize::try_from(tavern_pos.y))
                {
                    grid.add_clutter(x, y, BRAWL_CLUTTER);
                }
            }

            // Log it
            if let Some(ref mut l) = log {
                l.add_colored(
                    format!(
                        "A brawl broke out in a Tavern at ({}, {}) due to low morale!",
                        tavern_pos.x, tavern_pos.y
                    ),
                    ratatui::style::Color::Red,
                );
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(tavern_brawls_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_tavern_brawl_triggers() {
        let mut world = World::new();

        world.insert_resource(ClutterGrid::new(10, 10));

        // Create two miserable pops socializing
        let pop1 = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Socialize,
                    ..Default::default()
                },
                Morale {
                    value: 0.1,
                    ..Default::default()
                }, // Below threshold
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let pop2 = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Socialize,
                    ..Default::default()
                },
                Morale {
                    value: 0.1,
                    ..Default::default()
                }, // Below threshold
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let mut tavern = Tavern::default();
        tavern.visitors.push(pop1);
        tavern.visitors.push(pop2);

        world.spawn((tavern, GridPosition { x: 5, y: 5 }));

        // Run many times to ensure the 5% chance triggers
        for _ in 0..1000 {
            world.run_system_once(tavern_brawls_system).unwrap();
        }

        let health1 = world.get::<Health>(pop1).unwrap();
        assert!(
            health1.current < 100.0,
            "Pop 1 should have taken brawl damage"
        );

        let health2 = world.get::<Health>(pop2).unwrap();
        assert!(
            health2.current < 100.0,
            "Pop 2 should have taken brawl damage"
        );

        let grid = world.resource::<ClutterGrid>();
        assert!(grid.get(5, 5) > 0.0, "Brawl should create clutter");
    }
}
