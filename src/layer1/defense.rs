use crate::layer1::access_control::{AccessControl, AccessMode};
use crate::layer1::building::{Building, OccupiedTiles};
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainGrid;
use bevy_ecs::prelude::*;

/// Component representing a defense gate.
#[derive(Component, Default, Debug)]
pub struct Gate {
    /// Whether the gate is locked (impassable).
    pub is_locked: bool,
}

/// Checks if a tile is walkable (Terrain + Buildings).
///
/// This generic check assumes the agent has no special credentials.
/// It returns false for Restricted/Lockdown access controls.
#[allow(clippy::collapsible_if)]
pub fn is_walkable(world: &mut World, x: i32, y: i32) -> bool {
    // 1. Check Terrain
    let terrain = world.resource::<TerrainGrid>();
    if let (Ok(x_idx), Ok(y_idx)) = (usize::try_from(x), usize::try_from(y)) {
        if !terrain
            .get(x_idx, y_idx)
            .is_some_and(crate::layer1::terrain::TerrainType::is_walkable)
        {
            return false;
        }
    } else {
        return false; // Out of bounds
    }

    // 2. Check Buildings via OccupiedTiles
    if let Some(occupied) = world.get_resource::<OccupiedTiles>() {
        if occupied.0.contains(&(x, y)) {
            // Find the building entity at this position
            let mut blocked = false;
            let mut buildings = world.query::<(
                &GridPosition,
                &Building,
                Option<&Gate>,
                Option<&AccessControl>,
            )>();
            for (pos, building, gate, access_opt) in buildings.iter(world) {
                if pos.x == x && pos.y == y {
                    // Priority: AccessControl
                    if let Some(access) = access_opt {
                        match access.mode {
                            AccessMode::Public => {
                                // Walkable
                            }
                            _ => {
                                // Restricted or Lockdown -> Blocked for generic walker
                                blocked = true;
                            }
                        }
                    } else if let Some(g) = gate {
                        // Legacy Gate Logic
                        if g.is_locked {
                            blocked = true;
                        }
                    } else if building.building_type.is_obstacle() {
                        blocked = true;
                    }
                    break;
                }
            }
            if blocked {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::layer1::access_control::{AccessControl, AccessMode};
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::execution::MovementTarget;
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_types::ActionType;
    use bevy_ecs::prelude::*;

    // Helper to setup world with flat grass
    fn setup_world() -> World {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world
    }

    // 1. Building Obstacle Logic
    #[test]
    fn test_wall_is_obstacle() {
        let wall = Building {
            building_type: BuildingType::Wall,
        };
        assert!(wall.building_type.is_obstacle());
    }

    #[test]
    fn test_gate_is_obstacle_only_when_locked() {
        // Gate requires extra component or state in Building?
        // For MVP, Gate is a BuildingType. We might need a `Gate` component.
        let mut world = World::new();
        let gate_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Gate,
                },
                GridPosition { x: 0, y: 0 },
                crate::layer1::defense::Gate { is_locked: false },
            ))
            .id();

        // Check obstacle logic via system or helper
        assert!(!crate::layer1::defense::is_obstacle(&world, gate_entity));

        // Lock it
        world
            .get_mut::<crate::layer1::defense::Gate>(gate_entity)
            .unwrap()
            .is_locked = true;
        assert!(crate::layer1::defense::is_obstacle(&world, gate_entity));
    }

    #[test]
    fn test_access_control_overrides_obstacle() {
        let mut world = World::new();
        // Spawn a Gate (obstacle) with AccessControl::Public
        let gate = world
            .spawn((
                Building {
                    building_type: BuildingType::Gate,
                },
                GridPosition { x: 0, y: 0 },
                AccessControl {
                    mode: AccessMode::Public,
                    ..Default::default()
                },
            ))
            .id();

        assert!(!crate::layer1::defense::is_obstacle(&world, gate));

        // Change to Lockdown
        world.get_mut::<AccessControl>(gate).unwrap().mode = AccessMode::Lockdown;
        assert!(crate::layer1::defense::is_obstacle(&world, gate));
    }

    #[test]
    fn test_normal_buildings_are_obstacles() {
        // Most buildings should be obstacles (Housing, Farm?)
        // Spec decision: Farms are walkable? Housing is obstacle?
        // Let's say: Housing = Obstacle, Farm = Walkable (crops).
        let housing = Building {
            building_type: BuildingType::Housing,
        };
        assert!(housing.building_type.is_obstacle());

        let farm = Building {
            building_type: BuildingType::Farm,
        };
        assert!(!farm.building_type.is_obstacle()); // Farms are walkable
    }

    // 2. Pathfinding / Movement Logic
    #[test]
    fn test_movement_blocked_by_wall() {
        let mut world = setup_world();

        // Place Wall at (1, 0)
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 1, y: 0 },
            Health::default(),
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 0));

        // Pop at (0,0) trying to move to (2,0)
        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(999),
                    target_position: GridPosition { x: 2, y: 0 },
                    for_action: ActionType::Idle, // Just moving
                },
            ))
            .id();

        // Run movement
        // We need to register the system or call it.
        // But movement_system in execution.rs needs update to check obstacles.
        // For this test, we assume movement_system uses the new `is_walkable` logic.

        // We might need to inject the logic or update the system in place.
        // Since we are mocking, we can just call the updated function `is_walkable`.

        // Let's test `is_walkable` directly first.
        assert!(!crate::layer1::defense::is_walkable(&mut world, 1, 0));
        assert!(crate::layer1::defense::is_walkable(&mut world, 0, 0));
    }

    #[test]
    fn test_movement_allowed_through_open_gate() {
        let mut world = setup_world();

        // Gate at (1, 0)
        world.spawn((
            Building {
                building_type: BuildingType::Gate,
            },
            GridPosition { x: 1, y: 0 },
            crate::layer1::defense::Gate { is_locked: false },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 0));

        assert!(crate::layer1::defense::is_walkable(&mut world, 1, 0));
    }

    // 3. Building Health & Death
    #[test]
    fn test_building_takes_damage() {
        let mut health = Health {
            current: 100.0,
            max: 100.0,
        };
        health.take_damage(10.0);
        assert_eq!(health.current, 90.0);
    }

    #[test]
    fn test_building_death_message() {
        // Refactor death_system to distinguish pops vs buildings
        let mut world = World::new();
        world.insert_resource(crate::shared::log::MessageLog::default());

        let wall = world
            .spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                Health {
                    current: -1.0,
                    max: 100.0,
                },
            ))
            .id();

        crate::layer1::health::death_system(&mut world);

        assert!(world.get_entity(wall).is_err());

        let log = world.resource::<crate::shared::log::MessageLog>();
        // Should NOT say "Colonist has died"
        // Should say "Building destroyed" or nothing?
        // For now, check it doesn't say Colonist.
        // Implementation detail: we need to update death_system.

        // Note: The original spec test checked "DEATH: A colonist has died!" is NOT present.
        // I will check the actual new message.
        let msg = &log.messages.back().unwrap().text;
        assert_eq!(msg, "Building destroyed!");
    }
}

/// Helper to check if a specific building entity is an obstacle (locked gate or solid building).
pub fn is_obstacle(world: &World, entity: Entity) -> bool {
    if let Some(access) = world.get::<AccessControl>(entity) {
        return match access.mode {
            AccessMode::Public => false,
            _ => true, // Restricted/Lockdown are generic obstacles
        };
    }

    if let Some(gate) = world.get::<Gate>(entity) {
        return gate.is_locked;
    }
    if let Some(building) = world.get::<Building>(entity) {
        return building.building_type.is_obstacle();
    }
    false
}
