//! Pneumatic Tubes Logic.
//!
//! Systems for rapid item transport via pneumatic tubes.

use bevy_ecs::prelude::*;
use crate::layer1::inventory::{Inventory, InventoryItem};
use crate::layer1::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use std::collections::{HashMap, HashSet, VecDeque};

/// Terminal for sending/receiving items.
#[derive(Component, Debug, Clone)]
pub struct PneumaticTerminal {
    /// Unique ID of the terminal (for routing).
    pub id: u32,
    /// IDs of reachable terminals.
    pub connected_to: Vec<u32>,
}

/// A tube segment connecting terminals.
#[derive(Component, Debug, Clone)]
pub struct PneumaticTube;

/// An item carrier moving through the tube.
#[derive(Component, Debug, Clone)]
pub struct TubeCarrier {
    /// ID of the destination terminal.
    pub target_terminal_id: u32,
    /// The entity of the item being transported.
    pub item_entity: Entity,
    /// Current progress (distance traveled in tiles).
    pub progress: f32,
    /// Movement speed (tiles per tick).
    pub speed: f32,
    /// Cached path to the target.
    pub path: Option<Vec<GridPosition>>,
}

/// Marks a tube or terminal as clogged.
#[derive(Component, Debug, Clone)]
pub struct Clogged {
    /// Severity of the clog (0.0 - 1.0).
    pub severity: f32,
}

/// System to discover tube network connections.
pub fn tube_network_system(
    mut terminals: Query<(&mut PneumaticTerminal, &GridPosition)>,
    tubes: Query<&GridPosition, With<PneumaticTube>>,
) {
    let tube_positions: HashSet<GridPosition> = tubes.iter().copied().collect();

    // Map of Terminal ID to Position
    let mut term_positions = HashMap::new();
    for (term, pos) in &terminals {
        term_positions.insert(term.id, *pos);
    }

    let mut connectivity: HashMap<u32, Vec<u32>> = HashMap::new();

    for (term, start_pos) in &terminals {
        let mut reachable = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(*start_pos);
        visited.insert(*start_pos);

        while let Some(current_pos) = queue.pop_front() {
            for neighbor in get_neighbors(current_pos) {
                if visited.contains(&neighbor) {
                    continue;
                }

                let is_tube = tube_positions.contains(&neighbor);

                // Is it a terminal?
                let mut found_term_id = None;
                for (id, pos) in &term_positions {
                    if *pos == neighbor {
                        found_term_id = Some(*id);
                        break;
                    }
                }

                if let Some(id) = found_term_id {
                    if id != term.id {
                         reachable.push(id);
                    }
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                } else if is_tube {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
        connectivity.insert(term.id, reachable);
    }

    // Apply updates
    for (mut term, _) in &mut terminals {
        if let Some(conns) = connectivity.get(&term.id) {
            term.connected_to.clone_from(conns);
        }
    }
}

/// Moves carriers through the tube network.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
pub fn tube_transport_system(
    mut commands: Commands,
    mut carriers: Query<(Entity, &mut TubeCarrier, &GridPosition)>,
    tubes: Query<(&GridPosition, Option<&Clogged>), With<PneumaticTube>>,
    mut terminals: Query<(Entity, &PneumaticTerminal, &GridPosition, &mut Inventory)>,
    items: Query<&Item>,
) {
    let mut clogged_positions = HashSet::new();
    let mut tube_positions = HashSet::new();
    for (pos, clogged) in &tubes {
        tube_positions.insert(*pos);
        if clogged.is_some() {
            clogged_positions.insert(*pos);
        }
    }

    let mut term_map = HashMap::new();
    for (entity, term, pos, _) in &terminals {
        term_map.insert(term.id, (entity, *pos));
    }

    for (carrier_entity, mut carrier, start_pos) in &mut carriers {
        // If path not cached, compute it
        if carrier.path.is_none() {
            let target_id = carrier.target_terminal_id;
            if let Some((_, target_pos)) = term_map.get(&target_id) {
                 if let Some(p) = find_path(*start_pos, *target_pos, &tube_positions, &term_map) {
                     carrier.path = Some(p);
                 } else {
                     continue;
                 }
            } else {
                continue;
            }
        }

        let mut progress_update = 0.0;
        let mut arrived = false;

        if let Some(path) = &carrier.path {
             let current_dist = carrier.progress;
             let next_dist = current_dist + carrier.speed;

             let mut blocked = false;
             let mut actual_move = carrier.speed;

             let start_idx = current_dist.floor() as usize;
             let end_idx = next_dist.floor() as usize;

             for i in start_idx..=end_idx {
                 if i >= path.len() {
                     break;
                 }
                 let pos = path[i];
                 if clogged_positions.contains(&pos) {
                     blocked = true;
                     actual_move = 0.0;
                     break;
                 }
             }

             if !blocked {
                 progress_update = actual_move;
                 if current_dist + actual_move >= (path.len() as f32 - 1.0) {
                     arrived = true;
                 }
             }
        }

        if progress_update > 0.0 {
            carrier.progress += progress_update;
        }

        if arrived {
             if let Ok(item) = items.get(carrier.item_entity) {
                 let inv_item = InventoryItem {
                     item_type: item.item_type.clone(),
                 };

                 let target_id = carrier.target_terminal_id;
                 if let Some((target_entity, _)) = term_map.get(&target_id) {
                     if let Ok((_, _, _, mut inventory)) = terminals.get_mut(*target_entity) {
                         inventory.add(inv_item);
                     }
                 }

                 commands.entity(carrier_entity).remove::<TubeCarrier>();
                 commands.entity(carrier.item_entity).despawn();
             }
        }
    }
}

// Helper for Neighbors
fn get_neighbors(pos: GridPosition) -> Vec<GridPosition> {
    vec![
        GridPosition { x: pos.x + 1, y: pos.y },
        GridPosition { x: pos.x - 1, y: pos.y },
        GridPosition { x: pos.x, y: pos.y + 1 },
        GridPosition { x: pos.x, y: pos.y - 1 },
    ]
}

// Helper for BFS Pathfinding
fn find_path(
    start: GridPosition,
    end: GridPosition,
    tubes: &HashSet<GridPosition>,
    terminals: &HashMap<u32, (Entity, GridPosition)>,
) -> Option<Vec<GridPosition>> {
    let mut queue = VecDeque::new();
    let mut came_from: HashMap<GridPosition, GridPosition> = HashMap::new();

    queue.push_back(start);
    came_from.insert(start, start);

    while let Some(current) = queue.pop_front() {
        if current == end {
            let mut path = Vec::new();
            let mut curr = end;
            while curr != start {
                path.push(curr);
                curr = *came_from.get(&curr).unwrap();
            }
            path.push(start);
            path.reverse();
            return Some(path);
        }

        for neighbor in get_neighbors(current) {
             if came_from.contains_key(&neighbor) {
                 continue;
             }

             let is_tube = tubes.contains(&neighbor);
             let is_term = terminals.values().any(|(_, p)| *p == neighbor);

             if is_tube || is_term {
                 came_from.insert(neighbor, current);
                 queue.push_back(neighbor);
             }
        }
    }
    None
}

/// Handles clogging of tubes.
#[allow(clippy::missing_const_for_fn)]
pub fn tube_clog_system(
    _tubes: Query<&mut Clogged>,
) {
    // Placeholder
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::inventory::{Inventory, InventoryItem};
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_tube_network_connection() {
        let mut world = World::new();
        let term_a = world.spawn((
            PneumaticTerminal { id: 1, connected_to: vec![] },
            GridPosition { x: 0, y: 0 },
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        let _term_b = world.spawn((
            PneumaticTerminal { id: 2, connected_to: vec![] },
            GridPosition { x: 2, y: 0 },
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        world.spawn((PneumaticTube, GridPosition { x: 0, y: 0 }));
        world.spawn((PneumaticTube, GridPosition { x: 1, y: 0 }));
        world.spawn((PneumaticTube, GridPosition { x: 2, y: 0 }));

        let mut schedule = Schedule::default();
        schedule.add_systems(tube_network_system);
        schedule.run(&mut world);

        if let Some(a) = world.get::<PneumaticTerminal>(term_a) {
             assert!(a.connected_to.contains(&2));
        } else {
             panic!("Terminal A missing");
        }
    }

    #[test]
    fn test_tube_sends_item() {
        let mut world = World::new();
        let term_a = world.spawn((
            PneumaticTerminal { id: 1, connected_to: vec![2] },
            GridPosition { x: 0, y: 0 },
            Inventory::default(),
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        let term_b = world.spawn((
            PneumaticTerminal { id: 2, connected_to: vec![1] },
            GridPosition { x: 10, y: 0 },
            Inventory::default(),
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        for x in 0..=10 {
            world.spawn((PneumaticTube, GridPosition { x, y: 0 }));
        }

        world.get_mut::<Inventory>(term_a).unwrap().add(InventoryItem { item_type: ItemType::BuildingPermit });
        let permit_entity = world.spawn(Item { item_type: ItemType::BuildingPermit }).id();

        world.entity_mut(term_a).insert(TubeCarrier {
            target_terminal_id: 2,
            item_entity: permit_entity,
            progress: 0.0,
            speed: 2.0,
            path: None,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems((tube_network_system, tube_transport_system));

        // Run multiple times to cover distance
        for _ in 0..6 {
             schedule.run(&mut world);
        }

        let inv_b = world.get::<Inventory>(term_b).unwrap();
        assert!(!inv_b.items.is_empty(), "Inventory B should have received the item");
        assert_eq!(inv_b.items[0].item_type, ItemType::BuildingPermit);
    }

    #[test]
    fn test_tube_clogging() {
        let mut world = World::new();
        let term_a = world.spawn((
            PneumaticTerminal { id: 1, connected_to: vec![2] },
            GridPosition { x: 0, y: 0 },
            TubeCarrier {
                target_terminal_id: 2,
                item_entity: Entity::from_raw(0),
                progress: 0.0,
                speed: 2.0,
                path: None,
            },
            Inventory::default(),
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        let _term_b = world.spawn((
            PneumaticTerminal { id: 2, connected_to: vec![1] },
            GridPosition { x: 10, y: 0 },
            Inventory::default(),
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        for x in 0..=10 {
            world.spawn((PneumaticTube, GridPosition { x, y: 0 }));
        }

        world.spawn((PneumaticTube, GridPosition { x: 1, y: 0 }, super::Clogged { severity: 1.0 }));

        let mut schedule = Schedule::default();
        schedule.add_systems((tube_network_system, tube_transport_system));
        schedule.run(&mut world);

        let carrier = world.get::<TubeCarrier>(term_a).unwrap();
        assert_eq!(carrier.progress, 0.0, "Carrier should not move through clog");
    }
}
