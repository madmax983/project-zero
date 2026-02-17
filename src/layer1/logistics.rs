//! Conveyor and Hopper Logistics.
//!
//! Systems for automated transport and collection of resources.

use crate::layer1::building::{Building, BuildingType, Direction};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet};

/// Component for a conveyor belt that moves items.
#[derive(Component, Debug, Clone)]
pub struct ConveyorBelt {
    /// The direction items are moved.
    pub direction: Direction,
    /// The speed of movement (items moved per tick).
    pub speed: f32,
}

/// Component for a hopper that collects items into global storage.
#[derive(Component, Debug, Clone)]
pub struct Hopper;

/// Moves items that are on active conveyor belts.
#[allow(clippy::type_complexity, clippy::cast_sign_loss)]
pub fn conveyor_system(
    mut queries: ParamSet<(
        Query<(&GridPosition, &ConveyorBelt, &PowerConsumer)>,
        Query<(&GridPosition, &Building)>,
        Query<(Entity, &mut GridPosition), With<ResourceItem>>,
    )>,
    terrain: Res<TerrainGrid>,
) {
    // 1. Map active belts
    let mut belt_map = HashMap::new();
    for (pos, belt, power) in &queries.p0() {
        if power.active {
            belt_map.insert(*pos, belt.direction);
        }
    }

    if belt_map.is_empty() {
        return;
    }

    // 2. Map obstacles (Buildings that are obstacles, EXCEPT Hopper)
    let mut obstacles = HashSet::new();
    for (pos, building) in &queries.p1() {
        if building.building_type.is_obstacle() && building.building_type != BuildingType::Hopper {
            obstacles.insert(*pos);
        }
    }

    // 3. Move items
    for (_entity, mut pos) in &mut queries.p2() {
        if let Some(direction) = belt_map.get(&*pos) {
            let delta = direction.to_delta();
            let target_x = pos.x + delta.0;
            let target_y = pos.y + delta.1;

            // Check bounds
            if target_x < 0 || target_y < 0 {
                continue;
            }

            // Check terrain and obstacles
            if let Some(tile) = terrain.get(target_x as usize, target_y as usize) {
                // Check terrain blocking
                match tile {
                    TerrainType::Rock | TerrainType::Water => continue,
                    _ => {}
                }

                // Check building obstacles
                if obstacles.contains(&GridPosition {
                    x: target_x,
                    y: target_y,
                }) {
                    continue;
                }

                // Move item
                pos.x = target_x;
                pos.y = target_y;
            }
        }
    }
}

/// Collects items on active hoppers into colony resources.
pub fn hopper_system(
    mut commands: Commands,
    mut items: Query<(Entity, &mut ResourceItem, &GridPosition)>,
    hoppers: Query<(&GridPosition, &PowerConsumer), With<Hopper>>,
    mut resources: ResMut<ColonyResources>,
) {
    // 1. Collect active hopper positions
    let mut hopper_positions = HashSet::new();
    for (pos, power) in &hoppers {
        if power.active {
            hopper_positions.insert(*pos);
        }
    }

    if hopper_positions.is_empty() {
        return;
    }

    // 2. Consume items
    for (entity, mut item, pos) in &mut items {
        if hopper_positions.contains(pos) {
            // Determine how much fits
            let amount_to_add = item.amount;

            let (current, max) = match item.resource_type {
                ResourceType::Food => (resources.food, resources.max_food),
                ResourceType::Wood => (resources.wood, resources.max_wood),
                ResourceType::Stone => (resources.stone, resources.max_stone),
                ResourceType::Ore => (resources.ore, resources.max_ore),
                ResourceType::Metal => (resources.metal, resources.max_metal),
                ResourceType::Planks => (resources.planks, resources.max_planks),
                ResourceType::Blocks => (resources.blocks, resources.max_blocks),
                ResourceType::Waste => (resources.waste, resources.max_waste),
                ResourceType::Rations => (resources.rations, resources.max_rations),
                ResourceType::Fuel => (resources.fuel, resources.max_fuel),
            };

            let space = (max - current).max(0.0);
            let added = amount_to_add.min(space);

            if added > 0.0 {
                match item.resource_type {
                    ResourceType::Food => resources.add_food(added),
                    ResourceType::Wood => resources.add_wood(added),
                    ResourceType::Stone => resources.add_stone(added),
                    ResourceType::Ore => resources.add_ore(added),
                    ResourceType::Metal => resources.add_metal(added),
                    ResourceType::Planks => resources.add_planks(added),
                    ResourceType::Blocks => resources.add_blocks(added),
                    ResourceType::Waste => resources.add_waste(added),
                    ResourceType::Rations => resources.add_rations(added),
                    ResourceType::Fuel => resources.add_fuel(added),
                }

                item.amount -= added;
            }

            // Despawn if empty (or close enough)
            if item.amount < f32::EPSILON {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_conveyor_moves_item() {
        let mut world = World::new();
        // Setup Terrain
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Setup Conveyor at (0,0) facing East (Active)
        world.spawn((
            Building {
                building_type: BuildingType::ConveyorBelt,
            },
            ConveyorBelt {
                direction: Direction::East,
                speed: 1.0,
            },
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                demand: 1.0,
                active: true,
            },
        ));

        // Spawn Item at (0,0)
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 1.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run system
        let _ = world.run_system_once(conveyor_system);

        // Item should move to (1,0)
        let pos = world.get::<GridPosition>(item).unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_conveyor_needs_power() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Inactive Conveyor
        world.spawn((
            Building {
                building_type: BuildingType::ConveyorBelt,
            },
            ConveyorBelt {
                direction: Direction::East,
                speed: 1.0,
            },
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                demand: 1.0,
                active: false,
            }, // No power
        ));

        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 1.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let _ = world.run_system_once(conveyor_system);

        // Item should NOT move
        let pos = world.get::<GridPosition>(item).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_hopper_consumes_item() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default()); // Stone = 5.0
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Active Hopper at (0,0)
        world.spawn((
            Building {
                building_type: BuildingType::Hopper,
            },
            Hopper,
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                demand: 5.0,
                active: true,
            },
        ));

        // Item on Hopper
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 5.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let _ = world.run_system_once(hopper_system);

        // Item should be despawned
        assert!(world.get_entity(item).is_err());

        // Resources should increase (5.0 + 5.0 = 10.0)
        let res = world.resource::<ColonyResources>();
        assert!((res.stone - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hopper_respects_cap() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.max_stone = 10.0;
        res.stone = 9.0;
        world.insert_resource(res);
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Active Hopper
        world.spawn((
            Building {
                building_type: BuildingType::Hopper,
            },
            Hopper,
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                demand: 5.0,
                active: true,
            },
        ));

        // Item with amount 5.0
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 5.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let _ = world.run_system_once(hopper_system);

        // Should partial consume.
        // 9.0 + 5.0 = 14.0. Max 10.0. Consumed 1.0. Remaining item: 4.0.

        let item_comp = world.get::<ResourceItem>(item).unwrap();
        assert!((item_comp.amount - 4.0).abs() < f32::EPSILON);

        let res = world.resource::<ColonyResources>();
        assert!((res.stone - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_conveyor_blocked_by_obstacle() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Conveyor at 0,0 facing East
        world.spawn((
            Building {
                building_type: BuildingType::ConveyorBelt,
            },
            ConveyorBelt {
                direction: Direction::East,
                speed: 1.0,
            },
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                demand: 1.0,
                active: true,
            },
        ));

        // Wall at 1,0 (Obstacle)
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 1, y: 0 },
        ));

        // Item at 0,0
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 1.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let _ = world.run_system_once(conveyor_system);

        // Item should NOT move
        let pos = world.get::<GridPosition>(item).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_conveyor_feeds_hopper() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(ColonyResources::default()); // Stone 5.0

        // Conveyor at 0,0 facing East
        world.spawn((
            Building {
                building_type: BuildingType::ConveyorBelt,
            },
            ConveyorBelt {
                direction: Direction::East,
                speed: 1.0,
            },
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                demand: 1.0,
                active: true,
            },
        ));

        // Hopper at 1,0 (Obstacle, but should accept item)
        world.spawn((
            Building {
                building_type: BuildingType::Hopper,
            },
            Hopper,
            GridPosition { x: 1, y: 0 },
            PowerConsumer {
                demand: 5.0,
                active: true,
            },
        ));

        // Item at 0,0
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 5.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run conveyor system -> Move to 1,0
        let _ = world.run_system_once(conveyor_system);

        let pos = world.get::<GridPosition>(item).unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 0);

        // Run hopper system -> Consume
        let _ = world.run_system_once(hopper_system);

        assert!(world.get_entity(item).is_err());
        let res = world.resource::<ColonyResources>();
        assert!((res.stone - 10.0).abs() < f32::EPSILON);
    }
}
