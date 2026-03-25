//! Experimental module for The Dream Economy.
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::items::{Item, ItemType};
use crate::layer1::dreams::{DreamtThisSleep, DreamJournal};
use crate::layer1::stress::StressTracker;
use crate::shared::time::SimulationTime;

/// A building component that captures dreams and crystallizes them into items.
#[derive(Component, Debug, Clone, Default)]
pub struct DreamCatcher;

/// Marker component for a positive dream item.
#[derive(Component, Debug, Clone)]
pub struct DreamMoteItem;

/// Marker component for a negative dream item.
#[derive(Component, Debug, Clone)]
pub struct NightmareFragmentItem;


/// Harvesting system that listens for new dreams and creates items if near a DreamCatcher.
#[allow(clippy::type_complexity)]
pub fn harvest_dreams_system(
    mut commands: Commands,
    pop_query: Query<(&GridPosition, &DreamJournal), (With<Pop>, Added<DreamtThisSleep>)>,
    catcher_query: Query<&GridPosition, With<DreamCatcher>>,
) {
    for (pop_pos, journal) in &pop_query {
        if let Some(dream) = &journal.last_dream {
            // Look for nearby DreamCatcher
            for catcher_pos in &catcher_query {
                let dist = pop_pos.distance_chebyshev(*catcher_pos);
                // If within 2 grid distance
                if dist <= 2 {
                    let mut entity = commands.spawn((
                        Item { item_type: ItemType::None },
                        *catcher_pos,
                    ));
                    if dream.is_nightmare {
                        entity.insert(NightmareFragmentItem);
                    } else {
                        entity.insert(DreamMoteItem);
                    }
                    break; // Only harvest once per pop per night
                }
            }
        }
    }
}

/// Pops walking near a NightmareFragment gain stress.
pub fn nightmare_paranoia_system(
    mut pop_query: Query<(&GridPosition, &mut StressTracker), With<Pop>>,
    item_query: Query<(&GridPosition, &Item), With<NightmareFragmentItem>>,
    time: Res<SimulationTime>,
) {
    // To avoid applying paranoia too frequently, we can throttle it via tick.
    #[allow(clippy::manual_is_multiple_of)]
    if time.tick % 10 != 0 {
        return;
    }

    for (item_pos, _item) in &item_query {
        // Only run for items with NightmareFragmentItem
        if true {
            for (pop_pos, mut stress) in &mut pop_query {
                if pop_pos.distance_chebyshev(*item_pos) <= 3 {
                    // Gain 1.0 stress per tick (run every 10 ticks, so effectively 0.1 per tick)
                    stress.accumulated_stress += 1.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::dreams::Dream;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_harvest_dreams_system() {
        let mut world = World::new();

        // Add DreamCatcher
        world.spawn((
            DreamCatcher,
            GridPosition { x: 5, y: 5 },
            NightmareFragmentItem,
        ));

        // Add Pop with a good dream
        let journal = DreamJournal {
            last_dream: Some(Dream {
                content: "Good dream".to_string(),
                tick: 1,
                impact: 0.1,
                is_nightmare: false,
            }),
            history: vec![],
        };

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 6 },
            journal,
            DreamtThisSleep,
        ));

        world.run_system_once(harvest_dreams_system).unwrap();
        world.flush(); // Flush required to process commands and spawn the item

        // Should have spawned a DreamMote
        let mut query = world.query_filtered::<(&Item, &GridPosition), With<DreamMoteItem>>();
        let mut found_mote = false;
        for (_item, pos) in query.iter(&world) {
            if pos.x == 5 && pos.y == 5 {
                found_mote = true;
            }
        }
        assert!(found_mote, "DreamMote should have been spawned");
    }

    #[test]
    fn test_nightmare_paranoia_system() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 10, ..Default::default() }); // Tick must be % 10 == 0

        // Add Pop near NightmareFragment
        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 6 },
            StressTracker::default(),
        )).id();

        // Add NightmareFragment
        world.spawn((
            Item { item_type: ItemType::None },
            GridPosition { x: 5, y: 5 },
            NightmareFragmentItem,
        ));

        world.run_system_once(nightmare_paranoia_system).unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert_eq!(stress.accumulated_stress, 1.0, "Pop should have gained stress");
    }
}
