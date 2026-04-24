//! Shipbreaking System (Spec 409).
//!
//! The `Shipbreaking` system manages the process of dismantling crashed orbital ships to
//! salvage valuable resources. It generates work orders and handles yield calculations.

use bevy_ecs::prelude::*;

use crate::layer1::health::Health;
use crate::layer1::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ResourceItem, ResourceType};

// --- Components ---

/// Represents a piece of a crashed ship hull. It's indestructible by normal means.
#[derive(Component)]
pub struct HullTile;

/// Indicates this entity can be mined with a specific tool tier.
#[derive(Component)]
pub struct Mineable {
    pub required_tool_tier: u8,
}

/// What resources this entity drops when destroyed by mining.
#[derive(Component)]
pub struct YieldsOnMine(pub Vec<ResourceType>);

/// A miner capable of mining.
#[derive(Component)]
pub struct Miner {
    pub tool_tier: u8,
}

// --- Events ---

/// Event to spawn a crashed ship at a location.
#[derive(Event, Debug, Clone)]
pub struct SpawnCrashedShipEvent {
    pub location: crate::layer1::map::GridPosition,
}

/// Event representing a mining action.
#[derive(Event, Debug, Clone)]
pub struct MineEvent {
    pub miner: Entity,
    pub target: Entity,
}

// --- Systems ---

pub fn spawn_crashed_ship_system(
    mut events: EventReader<SpawnCrashedShipEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        // Just a simple multi-tile shape for tests
        let base_x = event.location.x;
        let base_y = event.location.y;

        let tiles = vec![
            (0, 0),
            (1, 0),
            (2, 0),
            (0, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ];

        for (dx, dy) in tiles {
            commands.spawn((
                HullTile,
                Mineable {
                    required_tool_tier: 3,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                YieldsOnMine(vec![ResourceType::Scrap]),
                GridPosition {
                    x: base_x + dx,
                    y: base_y + dy,
                },
            ));
        }
    }
}

pub fn mine_system(
    mut events: EventReader<MineEvent>,
    mut target_query: Query<(&Mineable, &mut Health)>,
    miner_query: Query<&Miner>,
) {
    for event in events.read() {
        if let Ok(miner) = miner_query.get(event.miner) {
            if let Ok((mineable, mut health)) = target_query.get_mut(event.target) {
                if miner.tool_tier >= mineable.required_tool_tier {
                    health.take_damage(10.0);
                }
            }
        }
    }
}

pub fn hull_destroyed_system(
    mut commands: Commands,
    query: Query<(Entity, &Health, &YieldsOnMine, &GridPosition), With<HullTile>>,
) {
    for (entity, health, yields, pos) in query.iter() {
        if health.current <= 0.0 {
            // Spawn yields
            for res_type in &yields.0 {
                commands.spawn((
                    ResourceItem {
                        resource_type: *res_type,
                        amount: 10.0, // Arbitrary amount
                    },
                    Item {
                        item_type: ItemType::None,
                    }, // Required by some systems
                    *pos,
                ));
            }
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingMap;
    use crate::layer1::building::OccupiedTiles;
    use crate::layer1::building::{try_place_building, BuildingType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_shipbreaking_hull_generation() {
        // Arrange
        let mut world = World::new();
        // ... setup terrain ...
        world.insert_resource(TerrainGrid {
            width: 100,
            height: 100,
            tiles: vec![TerrainType::Grass; 10000],
        });

        // Ensure events are registered
        world.init_resource::<Events<SpawnCrashedShipEvent>>();

        // Act
        // Spawn a crashed ship
        world.send_event(SpawnCrashedShipEvent {
            location: crate::layer1::map::GridPosition { x: 10, y: 10 },
        });

        // In a real app we'd call app.update(), here we just run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_crashed_ship_system);
        schedule.run(&mut world);

        // Assert
        // Check if multiple Hull tiles are spawned
        let hull_tiles: Vec<_> = world.query::<&HullTile>().iter(&world).collect();
        assert!(
            !hull_tiles.is_empty(),
            "Crashed ship should spawn Hull tiles"
        );
    }

    #[test]
    fn test_mining_hull_requires_tool() {
        // Arrange
        let mut world = World::new();
        world.init_resource::<Events<MineEvent>>();

        let hull_entity = world
            .spawn((
                HullTile,
                Mineable {
                    required_tool_tier: 3,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let miner_entity = world.spawn(Miner { tool_tier: 1 }).id();

        // Act
        // Attempt to mine
        world.send_event(MineEvent {
            miner: miner_entity,
            target: hull_entity,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(mine_system);
        schedule.run(&mut world);

        // Assert
        // Health should be unchanged
        let health = world
            .get::<Health>(hull_entity)
            .expect("Missing resource or component");
        assert_eq!(
            health.current, 100.0,
            "Miner with low tier tool should not damage Hull tile"
        );
    }

    #[test]
    fn test_mining_hull_yields_alloys() {
        // Arrange
        let mut world = World::new();
        world.init_resource::<Events<MineEvent>>();

        let hull_entity = world
            .spawn((
                HullTile,
                Mineable {
                    required_tool_tier: 3,
                },
                Health {
                    current: 10.0,
                    max: 100.0,
                    has_rust_lung: false,
                }, // Low health for easy mining
                YieldsOnMine(vec![ResourceType::Scrap]),
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let miner_entity = world.spawn(Miner { tool_tier: 3 }).id();

        // Act
        // Mine the tile
        world.send_event(MineEvent {
            miner: miner_entity,
            target: hull_entity,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems((mine_system, hull_destroyed_system).chain());
        schedule.run(&mut world);

        // Assert
        // Check for spawned resources or added to inventory
        // (Assuming a system handles drops)
        let drops: Vec<_> = world.query::<&ResourceItem>().iter(&world).collect();
        assert!(!drops.is_empty(), "Mining Hull should yield resources");
    }

    #[test]
    fn test_building_inside_hull() {
        // Arrange
        let mut world = World::new();

        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(BuildingMap::default());
        world.insert_resource(crate::layer1::resources::ColonyResources {
            wood: 100.0, // For cost of bed (assuming it costs wood like housing)
            ..Default::default()
        });

        // Spawn hull walls shaping a small room
        world.spawn((HullTile, GridPosition { x: 0, y: 0 }));
        world.spawn((HullTile, GridPosition { x: 1, y: 0 }));
        world.spawn((HullTile, GridPosition { x: 2, y: 0 }));
        // ... imagine a U shape

        // Act
        // Try placing a bed inside
        let placement_result = try_place_building(
            &mut world,
            1,
            1,
            BuildingType::Housing, // Using housing as a proxy for Bed
        );

        // Assert
        assert!(
            placement_result,
            "Should be able to place buildings inside hull boundaries"
        );
    }
}
