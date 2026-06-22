//! Orbital Harpoon Impacts and Resource Delivery
//!
//! This module handles the physical impact of orbital harpoons striking the planetary surface.
//! These massive kinetic projectiles deliver bulk resources to the colony, but cause localized
//! destruction (craters) and trigger seismic events upon impact.
//!
//! # Examples
//!
//! ```
//! use bevy_ecs::prelude::*;
//! use scale::layer1::physics::harpoon::HarpoonImpact;
//! use scale::layer1::core::map::GridPosition;
//! use scale::layer1::economy::resources::ResourceType;
//!
//! let mut world = World::new();
//! // Spawn an incoming harpoon payload
//! world.spawn(HarpoonImpact {
//!     position: GridPosition { x: 10, y: 10 },
//!     resource_type: ResourceType::Metal,
//!     amount: 1000.0,
//! });
//! ```
use crate::layer1::core::map::GridPosition;
use crate::layer1::economy::resources::{ResourceItem, ResourceType};
use crate::layer1::geology::GeologicalEvent;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct HarpoonImpact {
    pub position: GridPosition,
    pub resource_type: ResourceType,
    pub amount: f32,
}

pub fn process_harpoon_impact_system(
    mut commands: Commands,
    impacts: Query<(Entity, &HarpoonImpact)>,
    mut terrain_grid: Option<ResMut<TerrainGrid>>,
    mut quake_events: EventWriter<GeologicalEvent>,
) {
    for (entity, impact) in impacts.iter() {
        commands.spawn((
            ResourceItem {
                resource_type: impact.resource_type,
                amount: impact.amount,
            },
            GridPosition {
                x: impact.position.x,
                y: impact.position.y,
            },
        ));

        if let Some(ref mut grid) = terrain_grid {
            if impact.position.x >= 0 && impact.position.y >= 0 {
                grid.set(
                    impact.position.x as usize,
                    impact.position.y as usize,
                    TerrainType::Crater,
                );
            }
        }

        quake_events.send(GeologicalEvent {
            center: GridPosition {
                x: impact.position.x,
                y: impact.position.y,
            },
            magnitude: 5.0,
        });

        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_harpoon_impact_delivers_resources() {
        let mut world = World::new();
        world.init_resource::<Events<GeologicalEvent>>();
        let impact_event = world
            .spawn(HarpoonImpact {
                position: GridPosition { x: 10, y: 10 },
                resource_type: ResourceType::Metal,
                amount: 1000.0,
            })
            .id();

        world
            .run_system_once(process_harpoon_impact_system)
            .unwrap();

        let mut found = false;
        for (pos, resource) in world.query::<(&GridPosition, &ResourceItem)>().iter(&world) {
            if pos.x == 10 && pos.y == 10 {
                assert_eq!(resource.amount, 1000.0);
                assert_eq!(resource.resource_type, ResourceType::Metal);
                found = true;
            }
        }
        assert!(found);
        assert!(world.get_entity(impact_event).is_err());
    }

    #[test]
    fn test_harpoon_impact_destroys_tile_and_causes_quake() {
        let mut world = World::new();
        world.init_resource::<Events<GeologicalEvent>>();

        let terrain_grid = TerrainGrid {
            width: 20,
            height: 20,
            tiles: vec![TerrainType::Grass; 400],
        };
        world.insert_resource(terrain_grid);

        world.spawn(HarpoonImpact {
            position: GridPosition { x: 10, y: 10 },
            resource_type: ResourceType::Metal,
            amount: 1000.0,
        });

        world
            .run_system_once(process_harpoon_impact_system)
            .unwrap();

        let events = world.resource::<Events<GeologicalEvent>>();
        #[allow(deprecated)]
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();
        assert!(!emitted.is_empty());
        assert_eq!(emitted[0].center.x, 10);
        assert_eq!(emitted[0].center.y, 10);

        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(10, 10).unwrap(), TerrainType::Crater);
    }
}
    #[test]
    fn test_harpoon_impact_negative_bounds_do_not_panic() {
        let mut app = bevy::prelude::App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);
        let grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        app.insert_resource(grid);
        app.add_event::<GeologicalEvent>();

        // Spawn impact out of bounds (negative)
        app.world_mut().spawn(HarpoonImpact {
            position: GridPosition { x: -5, y: -5 },
            resource_type: crate::layer1::economy::ResourceType::Scrap,
            amount: 10.0,
        });

        app.add_systems(bevy::prelude::Update, process_harpoon_impact_system);

        // Should not panic on array index overflow wrapping
        app.update();
    }
