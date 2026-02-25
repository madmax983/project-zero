use crate::layer1::inventory::{Inventory, InventoryItem};
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Component for a Pneumatic Tube Terminal.
///
/// Acts as an entry/exit point for the tube network.
/// Carriers are launched from here.
#[derive(Component, Debug, Clone, Default)]
pub struct PneumaticTerminal {
    /// Unique identifier for the terminal.
    pub id: u32,
    /// List of reachable terminal IDs.
    pub connected_to: Vec<u32>,
}

/// Component marker for a Pneumatic Tube segment.
///
/// Carriers travel along entities with this component.
#[derive(Component, Debug, Clone, Default)]
pub struct PneumaticTube;

/// Component for a carrier traveling through the tube network.
#[derive(Component, Debug, Clone, Default)]
pub struct TubeCarrier {
    /// The ID of the destination terminal.
    pub target_terminal_id: u32,
    /// The item being transported.
    pub payload: InventoryItem,
    /// Current progress between tiles (0.0 to 1.0).
    pub progress: f32,
    /// Speed of movement (tiles per tick).
    pub speed: f32,
    /// Cached path to the target.
    pub path: Option<Vec<GridPosition>>,
}

/// Component indicating a blockage in a tube segment.
#[derive(Component, Debug, Clone, Default)]
pub struct Clogged {
    /// Severity of the clog (0.0 to 1.0). >0 blocks movement.
    pub severity: f32,
}

/// System that manages the movement of tube carriers.
///
/// Handles:
/// 1. Launching carriers from terminals (spawning independent entities).
/// 2. Pathfinding to the target terminal.
/// 3. Moving carriers along the path.
/// 4. Delivering items to the target terminal's inventory.
pub fn tube_transport_system(
    mut commands: Commands,
    launch_query: Query<(Entity, &TubeCarrier, &GridPosition), With<PneumaticTerminal>>,
    mut carriers: Query<
        (Entity, &mut TubeCarrier, &mut GridPosition),
        (Without<PneumaticTerminal>, Without<PneumaticTube>),
    >,
    tubes: Query<(&GridPosition, Option<&Clogged>), (With<PneumaticTube>, Without<TubeCarrier>)>,
    mut terminals: Query<(Entity, &GridPosition, &PneumaticTerminal, &mut Inventory)>,
) {
    // 1. Launch Logic
    for (entity, carrier, pos) in &launch_query {
        commands.spawn((
            TubeCarrier {
                target_terminal_id: carrier.target_terminal_id,
                payload: carrier.payload.clone(),
                progress: carrier.progress,
                speed: carrier.speed,
                path: None,
            },
            *pos,
        ));
        commands.entity(entity).remove::<TubeCarrier>();
    }

    // 2. Move Logic
    let mut tube_map: HashMap<GridPosition, bool> = HashMap::new();
    for (pos, clogged) in &tubes {
        let is_clogged = clogged.map_or(false, |c| c.severity > 0.0);
        tube_map.insert(*pos, is_clogged);
    }

    let mut term_id_to_entity: HashMap<u32, Entity> = HashMap::new();
    let mut term_id_to_pos: HashMap<u32, GridPosition> = HashMap::new();

    for (entity, pos, term, _) in &terminals {
        term_id_to_entity.insert(term.id, entity);
        term_id_to_pos.insert(term.id, *pos);
    }

    for (entity, mut carrier, mut pos) in &mut carriers {
        // Calculate path if needed
        if carrier.path.is_none() {
            if let Some(target_pos) = term_id_to_pos.get(&carrier.target_terminal_id) {
                if let Some(path) = find_path(*pos, *target_pos, &tube_map) {
                    carrier.path = Some(path);
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }

        let mut arrived = false;

        if let Some(mut path) = carrier.path.take() {
            if !path.is_empty() {
                let next_step = path[0];

                let is_clogged = *tube_map.get(&next_step).unwrap_or(&false);
                if is_clogged {
                    // Stuck, restore path
                    carrier.path = Some(path);
                    continue;
                }

                carrier.progress += carrier.speed;
                if carrier.progress >= 1.0 {
                    *pos = next_step;
                    path.remove(0);
                    carrier.progress = 0.0;
                }
            }

            if path.is_empty() {
                arrived = true;
            }

            // Restore path if not consumed/arrived (logic handles remove(0))
            // If path is empty, we restore it too, but 'arrived' flag handles cleanup
            carrier.path = Some(path);
        }

        if arrived {
            if let Some(target_entity) = term_id_to_entity.get(&carrier.target_terminal_id) {
                if let Ok((_, _, _, mut inv)) = terminals.get_mut(*target_entity) {
                    inv.add(carrier.payload.clone());
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn find_path(
    start: GridPosition,
    end: GridPosition,
    tube_map: &HashMap<GridPosition, bool>,
) -> Option<Vec<GridPosition>> {
    let mut queue = VecDeque::new();
    queue.push_back((start, vec![]));
    let mut visited = HashSet::new();
    visited.insert(start);

    while let Some((current, path)) = queue.pop_front() {
        if current == end {
            return Some(path);
        }

        let neighbors = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (dx, dy) in neighbors {
            let next = GridPosition {
                x: current.x + dx,
                y: current.y + dy,
            };

            if tube_map.contains_key(&next) && !visited.contains(&next) {
                visited.insert(next);
                let mut new_path = path.clone();
                new_path.push(next);
                queue.push_back((next, new_path));
            }
        }
    }
    None
}

/// System that updates the `connected_to` list for terminals based on network connectivity.
pub fn tube_network_system(
    mut terminals: Query<(&GridPosition, &mut PneumaticTerminal)>,
    tubes: Query<&GridPosition, With<PneumaticTube>>,
) {
    let mut tube_set = HashSet::new();
    for pos in &tubes {
        tube_set.insert(*pos);
    }

    let mut terminal_positions: HashMap<GridPosition, u32> = HashMap::new();
    for (pos, term) in &terminals {
        terminal_positions.insert(*pos, term.id);
    }

    for (start_pos, mut term) in &mut terminals {
        let reachable = find_reachable_terminals(*start_pos, &tube_set, &terminal_positions);
        term.connected_to = reachable;
    }
}

fn find_reachable_terminals(
    start: GridPosition,
    tube_set: &HashSet<GridPosition>,
    terminal_positions: &HashMap<GridPosition, u32>,
) -> Vec<u32> {
    let mut results = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(start);
    let mut visited = HashSet::new();
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        if current != start {
            if let Some(id) = terminal_positions.get(&current) {
                results.push(*id);
            }
        }

        let neighbors = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (dx, dy) in neighbors {
            let next = GridPosition {
                x: current.x + dx,
                y: current.y + dy,
            };

            if tube_set.contains(&next) && !visited.contains(&next) {
                visited.insert(next);
                queue.push_back(next);
            }
        }
    }
    results
}

/// System that manages tube clogging.
///
/// Currently a placeholder for future mechanics.
pub fn tube_clog_system(mut _tubes: Query<&mut Clogged>) {
    // Empty for MVP
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::inventory::{Inventory, InventoryItem};
    use crate::layer1::items::ItemType;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_tube_network_connection() {
        let mut world = World::new();
        let term_a = world
            .spawn((
                PneumaticTerminal {
                    id: 1,
                    connected_to: vec![],
                },
                GridPosition { x: 0, y: 0 },
                PowerConsumer {
                    active: true,
                    ..Default::default()
                },
            ))
            .id();

        let _term_b = world
            .spawn((
                PneumaticTerminal {
                    id: 2,
                    connected_to: vec![],
                },
                GridPosition { x: 2, y: 0 },
                PowerConsumer {
                    active: true,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((PneumaticTube, GridPosition { x: 0, y: 0 }));
        world.spawn((PneumaticTube, GridPosition { x: 1, y: 0 }));
        world.spawn((PneumaticTube, GridPosition { x: 2, y: 0 }));

        let mut schedule = Schedule::default();
        schedule.add_systems(tube_network_system);
        schedule.run(&mut world);

        let a = world.get::<PneumaticTerminal>(term_a).unwrap();
        assert!(a.connected_to.contains(&2));
    }

    #[test]
    fn test_tube_sends_item() {
        let mut world = World::new();
        let term_a = world
            .spawn((
                PneumaticTerminal {
                    id: 1,
                    connected_to: vec![2],
                },
                GridPosition { x: 0, y: 0 },
                Inventory::default(),
                PowerConsumer {
                    active: true,
                    ..Default::default()
                },
            ))
            .id();

        let term_b = world
            .spawn((
                PneumaticTerminal {
                    id: 2,
                    connected_to: vec![1],
                },
                GridPosition { x: 2, y: 0 },
                Inventory::default(),
                PowerConsumer {
                    active: true,
                    ..Default::default()
                },
            ))
            .id();

        let permit = InventoryItem {
            item_type: ItemType::BuildingPermit,
        };
        world
            .get_mut::<Inventory>(term_a)
            .unwrap()
            .add(permit.clone());
        world.get_mut::<Inventory>(term_a).unwrap().items.clear();

        // Manual insert simulates user/logic triggering send
        world.entity_mut(term_a).insert(TubeCarrier {
            target_terminal_id: 2,
            payload: permit.clone(),
            progress: 0.0,
            speed: 1.0,
            path: None,
        });

        world.spawn((PneumaticTube, GridPosition { x: 0, y: 0 }));
        world.spawn((PneumaticTube, GridPosition { x: 1, y: 0 }));
        world.spawn((PneumaticTube, GridPosition { x: 2, y: 0 }));

        let mut schedule = Schedule::default();
        schedule.add_systems(tube_transport_system);

        for _ in 0..10 {
            schedule.run(&mut world);
        }

        let inv_b = world.get::<Inventory>(term_b).unwrap();
        assert!(!inv_b.items.is_empty(), "Item should have arrived at B");
        assert_eq!(inv_b.items[0].item_type, ItemType::BuildingPermit);
    }

    #[test]
    fn test_tube_clogging() {
        let mut world = World::new();
        let term_a = world
            .spawn((
                PneumaticTerminal {
                    id: 1,
                    connected_to: vec![2],
                },
                GridPosition { x: 0, y: 0 },
                TubeCarrier {
                    target_terminal_id: 2,
                    payload: InventoryItem {
                        item_type: ItemType::None,
                    },
                    progress: 0.0,
                    speed: 1.0,
                    path: None,
                },
                Inventory::default(),
                PowerConsumer {
                    active: true,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            PneumaticTerminal {
                id: 2,
                connected_to: vec![],
            },
            GridPosition { x: 2, y: 0 },
            Inventory::default(),
            PowerConsumer {
                active: true,
                ..Default::default()
            },
        ));

        world.spawn((PneumaticTube, GridPosition { x: 0, y: 0 }));
        world.spawn((
            PneumaticTube,
            GridPosition { x: 1, y: 0 },
            Clogged { severity: 1.0 },
        ));
        world.spawn((PneumaticTube, GridPosition { x: 2, y: 0 }));

        let mut schedule = Schedule::default();
        schedule.add_systems(tube_transport_system);

        for _ in 0..5 {
            schedule.run(&mut world);
        }

        // Check original terminal for carrier (it was removed)
        assert!(world.get::<TubeCarrier>(term_a).is_none());

        // Find spawned carrier
        let mut carriers = world.query::<(&TubeCarrier, &GridPosition)>();
        let mut found = false;
        for (_c, pos) in carriers.iter(&world) {
            found = true;
            assert_eq!(pos.x, 0);
            assert_eq!(pos.y, 0);
        }
        assert!(found, "Carrier should exist");
    }
}
