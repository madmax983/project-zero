//! Hedonic Halo (Nova Feature).
//!
//! # The Spark
//! We have the `Needs` components (which track hunger, rest, leisure, etc.) and `Relationships` via `AffinityChange`.
//!
//! # The Feature
//! Pops that have completely full (or nearly full) needs act as a beacon of positivity.
//! When a pop's needs are all extremely high (>0.9), they passively project a "Hedonic Halo".
//! We iterate through all Pops, and if one has this halo, it emits a small `AffinityChange` increase
//! towards them for any Pop within a 2-tile radius.
//!
//! # The Potential
//! This mechanically rewards creating high-density "happy zones". Instead of just preventing negative
//! breakdowns, pushing Pops into ecstatic states naturally bonds the community together, turning
//! individual luxury into a social resource.

use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::social::AffinityChange;
use bevy_ecs::prelude::*;

const HALO_RADIUS: u32 = 2;
const HALO_AFFINITY_BOOST: f32 = 0.05;
const HALO_THRESHOLD: f32 = 0.9;

pub fn hedonic_halo_system(
    pops: Query<(Entity, &GridPosition, &Needs), With<Pop>>,
    mut affinity_events: EventWriter<AffinityChange>,
) {
    let mut happy_pops = Vec::new();

    // First, find all pops radiating the halo
    for (entity, pos, needs) in pops.iter() {
        if needs.hunger >= HALO_THRESHOLD
            && needs.rest >= HALO_THRESHOLD
            && needs.leisure >= HALO_THRESHOLD
            && needs.hygiene >= HALO_THRESHOLD
        {
            happy_pops.push((entity, *pos));
        }
    }

    if happy_pops.is_empty() {
        return;
    }

    // Now, emit affinity changes for neighbors
    for (listener_entity, listener_pos, _) in pops.iter() {
        for (happy_entity, happy_pos) in &happy_pops {
            if listener_entity != *happy_entity
                && listener_pos.distance_chebyshev(*happy_pos) <= HALO_RADIUS
            {
                affinity_events.send(AffinityChange {
                    source: listener_entity, // The listener's affinity for the happy pop changes
                    target: *happy_entity,
                    amount: HALO_AFFINITY_BOOST,
                });
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(hedonic_halo_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_hedonic_halo_emits_affinity() {
        let mut world = World::new();
        world.init_resource::<Events<AffinityChange>>();

        let happy_pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 0.95,
                    rest: 0.95,
                    leisure: 0.95,
                    hygiene: 0.95,
                },
            ))
            .id();

        let neighbor = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 6 }, // Distance 1
                Needs::default(),            // Doesn't matter
            ))
            .id();

        let _far_pop = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 }, // Distance 5
                Needs::default(),
            ))
            .id();

        world.run_system_once(hedonic_halo_system).unwrap();

        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();

        // Should only emit 1 event (from neighbor towards happy_pop)
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].source, neighbor);
        assert_eq!(emitted[0].target, happy_pop);
        assert_eq!(emitted[0].amount, HALO_AFFINITY_BOOST);
    }

    #[test]
    fn test_no_halo_if_one_need_low() {
        let mut world = World::new();
        world.init_resource::<Events<AffinityChange>>();

        let _pop1 = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 0.95,
                    rest: 0.95,
                    leisure: 0.1, // LOW
                    hygiene: 0.95,
                },
            ))
            .id();

        let _neighbor = world
            .spawn((Pop, GridPosition { x: 6, y: 6 }, Needs::default()))
            .id();

        world.run_system_once(hedonic_halo_system).unwrap();

        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).next().is_none());
    }
}
