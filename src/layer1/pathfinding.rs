use bevy_ecs::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::layer1::access_control::{AccessControl, AccessMode};
use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
use crate::layer1::control::{DoorControl, DoorState};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Role;
use crate::layer1::terrain::{TerrainGrid, TerrainType};

struct AccessCredentials {
    entity: Entity,
    role: Option<Role>,
}

type BuildingMap = HashMap<(i32, i32), (BuildingType, Option<AccessControl>, Option<DoorControl>)>;

/// Node for A* pathfinding.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Node {
    pos: (i32, i32),
    cost: i32,      // Accumulated cost (g)
    heuristic: i32, // Heuristic to target (h)
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap (lowest f = g + h first)
        (other.cost + other.heuristic)
            .cmp(&(self.cost + self.heuristic))
            .then_with(|| other.cost.cmp(&self.cost))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Finds a path between two points.
///
/// Returns `None` if no path is found.
///
/// # Arguments
///
/// * `world` - The Bevy World.
/// * `start` - Starting coordinates (x, y).
/// * `end` - Target coordinates (x, y).
pub fn find_path(world: &World, start: (i32, i32), end: (i32, i32)) -> Option<Vec<(i32, i32)>> {
    find_path_internal(world, start, end, false, None)
}

/// Finds a path for a specific pop, considering access control.
///
/// # Arguments
///
/// * `world` - The Bevy World.
/// * `start` - Starting coordinates.
/// * `end` - Target coordinates.
/// * `pop` - The pop entity.
pub fn find_path_for_pop(
    world: &World,
    start: (i32, i32),
    end: (i32, i32),
    pop: Entity,
) -> Option<Vec<(i32, i32)>> {
    let role = world.get::<Role>(pop).copied();
    let credentials = Some(AccessCredentials { entity: pop, role });
    find_path_internal(world, start, end, false, credentials.as_ref())
}

/// Finds a path for a specific entity, considering its capabilities.
///
/// # Arguments
///
/// * `world` - The Bevy World.
/// * `start` - Starting coordinates.
/// * `end` - Target coordinates.
/// * `_capability` - The entity capability component (e.g. Vermin).
pub fn find_path_for_entity<T: Component>(
    world: &World,
    start: (i32, i32),
    end: (i32, i32),
    _capability: &T,
) -> Option<Vec<(i32, i32)>> {
    let can_use_vents =
        std::any::TypeId::of::<T>() == std::any::TypeId::of::<crate::layer1::vermin::Vermin>();
    find_path_internal(world, start, end, can_use_vents, None)
}

fn find_path_internal(
    world: &World,
    start: (i32, i32),
    end: (i32, i32),
    can_use_vents: bool,
    credentials: Option<&AccessCredentials>,
) -> Option<Vec<(i32, i32)>> {
    let terrain = world.resource::<TerrainGrid>();
    let occupied = world.get_resource::<OccupiedTiles>();

    // Optimization: Collect building data into a map for fast lookups
    let mut building_map: BuildingMap = HashMap::new();
    for entity in world.iter_entities() {
        if let (Some(pos), Some(b)) = (entity.get::<GridPosition>(), entity.get::<Building>()) {
            let access = entity.get::<AccessControl>().cloned();
            let door = entity.get::<DoorControl>().copied();
            building_map.insert((pos.x, pos.y), (b.building_type, access, door));
        }
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut cost_so_far: HashMap<(i32, i32), i32> = HashMap::new();

    open_set.push(Node {
        pos: start,
        cost: 0,
        heuristic: manhattan_distance(start, end),
    });
    cost_so_far.insert(start, 0);

    while let Some(Node { pos, cost, .. }) = open_set.pop() {
        if pos == end {
            // Reconstruct path
            let mut path = Vec::new();
            let mut current = end;
            while current != start {
                path.push(current);
                current = *came_from.get(&current)?;
            }
            // Start is usually excluded or implicit in movement logic, but let's see what tests expect.
            // Tests check p.len() >= 2. Start -> Next -> End. Path should contain steps.
            path.reverse();
            return Some(path);
        }

        // Check neighbors (Manhattan)
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let next = (pos.0 + dx, pos.1 + dy);

            // Check if walkable
            if !is_walkable(
                next,
                terrain,
                occupied,
                &building_map,
                can_use_vents,
                credentials,
                next == end, // Ignore obstacle at target? Usually yes for "Interact", but for "MoveTo" maybe not.
                             // Greedy movement in execution.rs checks adjacency.
                             // find_path usually implies reaching the tile.
                             // But if target is a Wall/Vent, we can't stand inside it unless we can pass through.
                             // For Vermin -> Vent, they CAN pass through, so it is walkable.
                             // For Pop -> Wall, they cannot.
                             // So we should NOT ignore obstacle at target unless logic allows.
            ) {
                continue;
            }

            // Movement cost (default 1 + terrain cost)
            let tile_cost =
                if let (Ok(x), Ok(y)) = (usize::try_from(next.0), usize::try_from(next.1)) {
                    #[allow(clippy::cast_possible_truncation)]
                    terrain.get(x, y).map_or(1, |t| t.movement_cost() as i32)
                } else {
                    1
                };

            let new_cost = cost + tile_cost;

            if new_cost < *cost_so_far.get(&next).unwrap_or(&i32::MAX) {
                cost_so_far.insert(next, new_cost);
                let priority = new_cost + manhattan_distance(next, end);
                open_set.push(Node {
                    pos: next,
                    cost: new_cost,
                    heuristic: priority,
                });
                came_from.insert(next, pos);
            }
        }
    }

    None
}

const fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn is_walkable(
    pos: (i32, i32),
    terrain: &TerrainGrid,
    occupied: Option<&OccupiedTiles>,
    building_map: &BuildingMap,
    can_use_vents: bool,
    credentials: Option<&AccessCredentials>,
    _is_target: bool,
) -> bool {
    let (x, y) = pos;

    // 1. Check Bounds & Terrain
    let Ok(ux) = usize::try_from(x) else {
        return false;
    };
    let Ok(uy) = usize::try_from(y) else {
        return false;
    };

    if !terrain.get(ux, uy).is_some_and(TerrainType::is_walkable) {
        return false;
    }

    // 2. Check Occupied Tiles
    if let Some(occ) = occupied {
        if !occ.0.contains(&(x, y)) {
            return true;
        }
    } else {
        return true;
    }

    // 3. Check Building Type
    if let Some(&(building_type, ref access_opt, ref door_opt)) = building_map.get(&(x, y)) {
        // Check Door Control first (physical state overrides)
        if let Some(door) = door_opt {
            match door.state {
                DoorState::Locked => return false,
                DoorState::Open => return true,
                DoorState::Auto => { /* Continue to check AccessControl */ }
            }
        }

        if let Some(access) = access_opt {
            return match access.mode {
                AccessMode::Public => true,
                AccessMode::Lockdown => false,
                AccessMode::Restricted => credentials.is_some_and(|creds| {
                    access.allowed_pops.contains(&creds.entity)
                        || creds
                            .role
                            .is_some_and(|r| access.allowed_roles.contains(&r))
                }),
            };
        }

        if building_type == BuildingType::Vent && can_use_vents {
            return true;
        }

        if building_type.is_obstacle() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::vermin::Vermin;

    fn setup_world() -> World {
        let mut world = World::new();
        // 10x10 grass
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world
    }

    #[test]
    fn test_find_path_simple() {
        let world = setup_world();
        let path = find_path(&world, (0, 0), (2, 0));
        assert!(path.is_some(), "Should find simple straight path");
        let p = path.unwrap();
        // Path from (0,0) to (2,0) should be [(1,0), (2,0)] or similar length
        assert!(p.len() >= 2);
        assert_eq!(p.last(), Some(&(2, 0)));
    }

    #[test]
    fn test_vent_blocks_pop_movement() {
        let mut world = setup_world();

        // Vent at (1, 0)
        world.spawn((
            Building {
                building_type: BuildingType::Vent,
            },
            GridPosition { x: 1, y: 0 },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 0));

        // Pop at (0, 0) trying to move to (2, 0)
        // Check pathfinding
        let path = find_path(&world, (0, 0), (2, 0));
        // If it's blocked, it might find a long way around (if map is open)
        // But let's assume we blocked the only way or check if it avoids (1,0).
        // Since map is 10x10 grass, it will just walk around (0,1) -> (1,1) -> (2,1) -> (2,0).

        if let Some(p) = path {
            assert!(
                !p.contains(&(1, 0)),
                "Path should NOT contain Vent at (1,0)"
            );
        }

        // Let's make a corridor to force block
        // Walls at (1, -1) and (1, 1)? Map is 0..10.
        // Wall at (1, 1). Edge is y=0.
        // So (1,0) is the choke point if we block (1,1).
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 1, y: 1 },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 1));

        // Wall at (0, 1) and (2, 1) too to be sure?
        // Actually, let's just assert that if we ask to path THROUGH it, it fails if it's the only option.
        // Or simpler: check if `is_walkable` logic (if exposed) works.
        // But `find_path` is high level.

        // Let's create a map where (1,0) is the only bridge.
        // W W W
        // S V E
        // W W W
        // Start (0,1), Vent (1,1), End (2,1). Walls at y=0 and y=2.

        let mut world = setup_world();
        // Block rows 0 and 2
        for x in 0..3 {
            world.spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x, y: 0 },
            ));
            world.resource_mut::<OccupiedTiles>().0.insert((x, 0));
            world.spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x, y: 2 },
            ));
            world.resource_mut::<OccupiedTiles>().0.insert((x, 2));
        }

        // Vent at (1, 1)
        world.spawn((
            Building {
                building_type: BuildingType::Vent,
            },
            GridPosition { x: 1, y: 1 },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 1));

        let path = find_path(&world, (0, 1), (2, 1));
        assert!(
            path.is_none(),
            "Pop should not path through Vent when it is the only way"
        );
    }

    #[test]
    fn test_vermin_passes_through_vent() {
        let mut world = setup_world();

        // Block rows 0 and 2
        for x in 0..3 {
            world.spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x, y: 0 },
            ));
            world.resource_mut::<OccupiedTiles>().0.insert((x, 0));
            world.spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x, y: 2 },
            ));
            world.resource_mut::<OccupiedTiles>().0.insert((x, 2));
        }

        // Vent at (1, 1)
        world.spawn((
            Building {
                building_type: BuildingType::Vent,
            },
            GridPosition { x: 1, y: 1 },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 1));

        // Vermin at (0, 1)
        // Check pathfinding with Vermin capability
        let path = find_path_for_entity(&world, (0, 1), (2, 1), &Vermin);
        assert!(path.is_some(), "Vermin SHOULD path through Vent");
    }

    #[test]
    fn test_locked_door_blocks_path() {
        let mut world = setup_world();

        // Block rows 0 and 2
        for x in 0..3 {
            world.spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x, y: 0 },
            ));
            world.resource_mut::<OccupiedTiles>().0.insert((x, 0));
            world.spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x, y: 2 },
            ));
            world.resource_mut::<OccupiedTiles>().0.insert((x, 2));
        }

        // Gate at (1, 1) - Public Access but Locked DoorControl
        world.spawn((
            Building {
                building_type: BuildingType::Gate,
            },
            GridPosition { x: 1, y: 1 },
            AccessControl {
                mode: AccessMode::Public,
                ..Default::default()
            },
            DoorControl {
                state: DoorState::Locked,
            },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 1));

        // Try to path from (0, 1) to (2, 1)
        let path = find_path(&world, (0, 1), (2, 1));
        assert!(
            path.is_none(),
            "Locked door should block path even if Public"
        );
    }

    #[test]
    fn test_open_door_allows_path_restricted() {
        use crate::layer1::pop::Pop;
        let mut world = setup_world();

        // Block rows 0 and 2
        for x in 0..3 {
            world.spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x, y: 0 },
            ));
            world.resource_mut::<OccupiedTiles>().0.insert((x, 0));
            world.spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x, y: 2 },
            ));
            world.resource_mut::<OccupiedTiles>().0.insert((x, 2));
        }

        // Gate at (1, 1) - Lockdown Access but Open DoorControl
        world.spawn((
            Building {
                building_type: BuildingType::Gate,
            },
            GridPosition { x: 1, y: 1 },
            AccessControl {
                mode: AccessMode::Lockdown,
                ..Default::default()
            },
            DoorControl {
                state: DoorState::Open,
            },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 1));

        let pop = world.spawn(Pop).id();

        // Try to path from (0, 1) to (2, 1)
        let path = crate::layer1::pathfinding::find_path_for_pop(&world, (0, 1), (2, 1), pop);
        assert!(
            path.is_some(),
            "Open door should allow path even if Lockdown"
        );
    }
}
