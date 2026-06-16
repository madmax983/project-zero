//! Conveyor and Hopper Logistics.
//!
//! Systems for automated transport and collection of resources.

use crate::layer1::building::{Building, BuildingType, Direction};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ColonyResources, ResourceItem};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy::utils::{HashMap, HashSet};
use bevy_ecs::prelude::*;

/// The physical variant of a conveyor belt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BeltVariant {
    #[default]
    Standard,
    Underground,
    Overhead,
}

/// Component for a conveyor belt that moves items.
#[derive(Component, Debug, Clone)]
pub struct ConveyorBelt {
    /// The direction items are moved.
    pub direction: Direction,
    /// The speed of movement (items moved per tick).
    pub speed: f32,
    /// The physical placement variant of the belt.
    pub variant: BeltVariant,
}

impl ConveyorBelt {
    /// Determines whether this conveyor belt blocks pathfinding for pops.
    pub fn blocks_pathfinding(&self) -> bool {
        match self.variant {
            BeltVariant::Standard => true,
            BeltVariant::Underground | BeltVariant::Overhead => false,
        }
    }
}

/// Component for a hopper that collects items into global storage.
#[derive(Component, Debug, Clone)]
pub struct Hopper;

#[derive(Component, Debug, Clone)]
pub struct Inserter {
    pub pickup_direction: Direction,
    pub dropoff_direction: Direction,
}

/// Moves items that are on active conveyor belts.
///
/// ⚡ Bolt Optimization: Uses `bevy::utils::HashMap` and `HashSet` (AHash) instead of `std::collections` equivalents to eliminate SipHash overhead on grid positions.
#[allow(clippy::type_complexity, clippy::cast_sign_loss)]
pub fn conveyor_system(
    mut queries: ParamSet<(
        Query<(&GridPosition, &ConveyorBelt, &PowerConsumer)>,
        Query<(&GridPosition, &Building)>,
        Query<(Entity, &mut GridPosition), With<ResourceItem>>,
    )>,
    terrain: Res<TerrainGrid>,
) {
    let mut belt_map = HashMap::new();
    for (pos, belt, power) in &queries.p0() {
        if power.active {
            belt_map.insert(*pos, belt.direction);
        }
    }
    if belt_map.is_empty() {
        return;
    }

    let mut obstacles = HashSet::new();
    for (pos, building) in &queries.p1() {
        if building.building_type.is_obstacle() && building.building_type != BuildingType::Hopper {
            obstacles.insert(*pos);
        }
    }

    let mut current_positions = HashSet::new();
    let mut moving_items = Vec::new();

    for (entity, pos) in &queries.p2() {
        if let Some(direction) = belt_map.get(pos) {
            let delta = direction.to_delta();
            let target_x = pos.x + delta.0;
            let target_y = pos.y + delta.1;
            moving_items.push((
                entity,
                *pos,
                GridPosition {
                    x: target_x,
                    y: target_y,
                },
            ));
        } else {
            current_positions.insert(*pos);
        }
    }

    let mut target_positions = HashMap::new();
    for (_, _, target) in &moving_items {
        *target_positions.entry(*target).or_insert(0) += 1;
    }

    for (entity, mut pos) in &mut queries.p2() {
        let mut new_pos = None;
        for (moving_entity, _, target_pos) in &moving_items {
            if *moving_entity == entity && target_pos.x >= 0 && target_pos.y >= 0 {
                if let Some(tile) = terrain.get(target_pos.x as usize, target_pos.y as usize) {
                    if !matches!(tile, TerrainType::Rock | TerrainType::Water)
                        && !obstacles.contains(target_pos)
                        && !current_positions.contains(target_pos)
                        && target_positions.get(target_pos) == Some(&1)
                    {
                        new_pos = Some(*target_pos);
                    }
                }
            }
        }
        if let Some(np) = new_pos {
            *pos = np;
        }
    }
}

/// Collects items on active hoppers into colony resources.
///
/// ⚡ Bolt Optimization: Uses `bevy::utils::HashMap` and `HashSet` (AHash) instead of `std::collections` equivalents to eliminate SipHash overhead on grid positions.
#[allow(clippy::type_complexity)]
pub fn inserter_system(
    inserters: Query<(&GridPosition, &Inserter, &PowerConsumer)>,
    mut items: Query<(Entity, &mut GridPosition), (With<ResourceItem>, Without<Inserter>)>,
) {
    let mut active_inserters = HashMap::new();
    for (pos, inserter, power) in &inserters {
        if power.active {
            active_inserters.insert(*pos, inserter);
        }
    }
    if active_inserters.is_empty() {
        return;
    }

    let mut taken = HashSet::new();
    for (pos, inserter) in active_inserters {
        let pickup_pos = GridPosition {
            x: pos.x + inserter.pickup_direction.to_delta().0,
            y: pos.y + inserter.pickup_direction.to_delta().1,
        };
        let dropoff_pos = GridPosition {
            x: pos.x + inserter.dropoff_direction.to_delta().0,
            y: pos.y + inserter.dropoff_direction.to_delta().1,
        };

        for (entity, mut item_pos) in &mut items {
            if *item_pos == pickup_pos && !taken.contains(&entity) {
                item_pos.x = dropoff_pos.x;
                item_pos.y = dropoff_pos.y;
                taken.insert(entity);
                break;
            }
        }
    }
}

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

            let current = resources.get_amount(item.resource_type);
            let max = resources.get_max_amount(item.resource_type);

            let space = (max - current).max(0.0);
            let added = amount_to_add.min(space);

            if added > 0.0 {
                resources.add_resource(&item.resource_type, added);

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
    use crate::layer1::resources::ResourceType;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_conveyor_blocks_pathfinding() {
        let conveyor = ConveyorBelt {
            direction: Direction::East,
            speed: 1.0,
            variant: BeltVariant::Standard,
        };
        assert!(
            conveyor.blocks_pathfinding(),
            "Standard conveyors should block pathfinding"
        );

        let underground = ConveyorBelt {
            direction: Direction::East,
            speed: 1.0,
            variant: BeltVariant::Underground,
        };
        assert!(
            !underground.blocks_pathfinding(),
            "Underground conveyors should not block pathfinding"
        );

        let overhead = ConveyorBelt {
            direction: Direction::East,
            speed: 1.0,
            variant: BeltVariant::Overhead,
        };
        assert!(
            !overhead.blocks_pathfinding(),
            "Overhead conveyors should not block pathfinding"
        );
    }

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
                variant: BeltVariant::Standard,
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
        let _ = world.run_system_once(super::conveyor_system);

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
                variant: BeltVariant::Standard,
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

        let _ = world.run_system_once(super::conveyor_system);

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
        let res = ColonyResources {
            max_stone: 10.0,
            stone: 9.0,
            ..Default::default()
        };
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
                variant: BeltVariant::Standard,
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

        let _ = world.run_system_once(super::conveyor_system);

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
                variant: BeltVariant::Standard,
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
        let _ = world.run_system_once(super::conveyor_system);

        let pos = world.get::<GridPosition>(item).unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 0);

        // Run hopper system -> Consume
        let _ = world.run_system_once(hopper_system);

        assert!(world.get_entity(item).is_err());
        let res = world.resource::<ColonyResources>();
        assert!((res.stone - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inserter_pickup_dropoff() {
        let mut world = World::new();
        world.spawn((
            super::Inserter {
                pickup_direction: Direction::West,
                dropoff_direction: Direction::East,
            },
            GridPosition { x: 1, y: 0 },
            PowerConsumer {
                demand: 1.0,
                active: true,
            },
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
        let _ = world.run_system_once(super::inserter_system);
        assert_eq!(world.get::<GridPosition>(item).unwrap().x, 2);
    }

    #[test]
    fn test_conveyor_overlap_prevention() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        world.spawn((
            Building {
                building_type: BuildingType::ConveyorBelt,
            },
            ConveyorBelt {
                direction: Direction::East,
                speed: 1.0,
                variant: BeltVariant::Standard,
            },
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                demand: 1.0,
                active: true,
            },
        ));
        world.spawn((
            Building {
                building_type: BuildingType::ConveyorBelt,
            },
            ConveyorBelt {
                direction: Direction::North,
                speed: 1.0,
                variant: BeltVariant::Standard,
            },
            GridPosition { x: 1, y: 1 },
            PowerConsumer {
                demand: 1.0,
                active: true,
            },
        ));

        let item1 = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 1.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();
        let item2 = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 1.0,
                },
                GridPosition { x: 1, y: 1 },
            ))
            .id();

        let _ = world.run_system_once(super::conveyor_system);

        assert_eq!(world.get::<GridPosition>(item1).unwrap().x, 0);
        assert_eq!(world.get::<GridPosition>(item2).unwrap().y, 1);
    }
}
