#![allow(clippy::type_complexity)]
//! The Haunted Cartographer (Nova Feature).
//!
//! # The Spark
//! We track where Pops die with `PopDied` events. What if the tiles where pops die become "Haunted"?
//!
//! # The Feature
//! A system that tracks how many Pops have died on each grid tile (`HauntedGrid`). If a Pop's current `Action` is `ActionType::Explore` and they walk onto a highly haunted tile, they have a chance to enter a `Breakdown` (Panic mental break).
//! This ties the historical legacy of the colony's failures directly to the spatial exploration loop, creating a mechanic where the map itself becomes a terrifying record of past mistakes.

use crate::layer1::entities::pop::{Pop, PopDied};
use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::ActionType;
use crate::layer1::psychology::stress::{Breakdown, BreakdownType};
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashMap;

/// A resource that tracks the number of deaths on each tile.
#[derive(Resource, Default)]
pub struct HauntedGrid {
    pub death_counts: HashMap<GridPosition, u32>,
}

/// System that records pop deaths onto the HauntedGrid.
pub fn record_haunted_tiles_system(
    mut events: EventReader<PopDied>,
    mut haunted_grid: ResMut<HauntedGrid>,
    // Removed dependency on querying the Pop entity directly as it might have been despawned
    // The GridPosition should ideally be carried on the event itself, but without changing core
    // logic we fallback to seeing if we can query it, or assume death location by other means.
    // However, PopDied event doesn't carry position.
    // We will query the entity BEFORE despawn if possible, assuming ordering allows it.
    pops: Query<&GridPosition, With<Pop>>,
) {
    for event in events.read() {
        if let Ok(pos) = pops.get(event.entity) {
            let count = haunted_grid.death_counts.entry(*pos).or_insert(0);
            *count += 1;
        }
    }
}

/// System that checks if exploring pops step on haunted tiles and triggers breakdowns.
pub fn haunted_exploration_system(
    mut commands: Commands,
    haunted_grid: Res<HauntedGrid>,
    query: Query<
        (
            Entity,
            &GridPosition,
            &crate::layer1::mind::utility_types::PopAction,
        ),
        (With<Pop>, Without<Breakdown>),
    >,
) {
    let mut rng = rand::thread_rng();

    for (entity, pos, action) in query.iter() {
        if action.current == ActionType::Explore {
            if let Some(&death_count) = haunted_grid.death_counts.get(pos) {
                // Base chance is 5% per death on the tile
                let chance = (death_count as f32 * 0.05).clamp(0.0, 0.5); // Max 50% chance per tick

                if rng.gen::<f32>() < chance {
                    commands.entity(entity).insert(Breakdown {
                        breakdown_type: BreakdownType::Dazing, // Panic!
                        duration_remaining: 100,               // 100 ticks of dazing
                    });
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        record_haunted_tiles_system
            .before(crate::layer1::biology::health::despawn_dead_entities_system),
        haunted_exploration_system,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_record_haunted_tiles() {
        let mut world = World::new();
        world.init_resource::<Events<PopDied>>();
        world.init_resource::<HauntedGrid>();

        let pop = world.spawn((Pop, GridPosition { x: 5, y: 5 })).id();

        world.send_event(PopDied {
            entity: pop,
            name: "Test".to_string(),
            tick: 1,
            reason: "Testing".to_string(),
        });

        world.run_system_once(record_haunted_tiles_system).unwrap();

        let grid = world.resource::<HauntedGrid>();
        assert_eq!(
            *grid.death_counts.get(&GridPosition { x: 5, y: 5 }).unwrap(),
            1
        );
    }
}
