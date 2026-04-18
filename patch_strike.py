import sys

content = """use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::events::BuildingRemovedEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::resources::ResourceType;
use bevy_ecs::prelude::*;
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct SubsurfaceResource {
    pub kind: ResourceType,
    pub depth: f32,
}

#[derive(Event, Debug, Clone)]
pub struct KineticStrikeEvent {
    pub target_x: i32,
    pub target_y: i32,
    pub accuracy_offset: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn kinetic_strike_system(
    mut commands: Commands,
    mut events: EventReader<KineticStrikeEvent>,
    mut grid: ResMut<TerrainGrid>,
    mut tile_query: Query<(Entity, &SubsurfaceResource, &GridPosition)>,
    mut building_removed_events: EventWriter<BuildingRemovedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    building_map: Res<crate::layer1::building::BuildingMap>,
    buildings: Query<(Entity, &crate::layer1::building::Building)>,
) {
    for event in events.read() {
        let actual_hit_x = event.target_x;
        let actual_hit_y = event.target_y;

        // Applying blast radius effect
        let blast_radius: i32 = 1;

        let mut hit_magma = false;
        let mut hit_ore = false;

        for (res_entity, resource, pos) in tile_query.iter_mut() {
            if (pos.x - actual_hit_x).abs() <= blast_radius && (pos.y - actual_hit_y).abs() <= blast_radius {
                // Determine resource kind hit
                match resource.kind {
                    ResourceType::Ore => {
                        hit_ore = true;
                        // Expose ore crater
                        if pos.x >= 0 && pos.x < grid.width as i32 && pos.y >= 0 && pos.y < grid.height as i32 {
                            grid.set(pos.x as usize, pos.y as usize, TerrainType::Dirt);
                        }
                    }
                    ResourceType::Fuel => {
                        hit_magma = true;
                        if pos.x >= 0 && pos.x < grid.width as i32 && pos.y >= 0 && pos.y < grid.height as i32 {
                            grid.set(pos.x as usize, pos.y as usize, TerrainType::MagmaRock);
                        }
                    }
                    _ => {}
                }

                // Destroy resource node entity if needed (optional)
                commands.entity(res_entity).despawn();
            }
        }

        // Destroy buildings in blast radius
        for dy in -blast_radius..=blast_radius {
            for dx in -blast_radius..=blast_radius {
                let x = actual_hit_x + dx;
                let y = actual_hit_y + dy;

                if x >= 0 && x < grid.width as i32 && y >= 0 && y < grid.height as i32 {
                    if let Some(&building_entity) = building_map.0.get(&(x, y)) {
                        if let Ok((entity, building)) = buildings.get(building_entity) {
                            building_removed_events.send(BuildingRemovedEvent {
                                entity,
                                position: GridPosition { x, y },
                                building_type: building.building_type,
                            });
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
        }

        // Trigger chronicle event
        if hit_magma {
            chronicle_events.send(AddChronicleEvent {
                importance: EventImportance::Major,
                text: "The Kinetic Strike missed its target and struck a deep magma layer, creating an active volcano!".to_string(),
            });
        } else if hit_ore {
            chronicle_events.send(AddChronicleEvent {
                importance: EventImportance::Standard,
                text: "The Colony Called Down the Thunder. The Kinetic Strike successfully exposed the ore veins.".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingMap;

    #[test]
    fn test_kinetic_strike_exposes_ore() {
        let mut app = App::new();
        app.add_systems(Update, kinetic_strike_system);

        // Setup real dependencies
        app.world_mut().insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        app.world_mut().insert_resource(BuildingMap::default());
        app.add_event::<BuildingRemovedEvent>();
        app.add_event::<AddChronicleEvent>();

        app.world_mut().spawn((
            SubsurfaceResource { kind: ResourceType::Ore, depth: 50.0 },
            GridPosition { x: 5, y: 5 },
        ));

        app.add_event::<KineticStrikeEvent>();
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 5,
            target_y: 5,
            accuracy_offset: 0.0,
        });

        app.update();

        let grid = app.world().get_resource::<TerrainGrid>().unwrap();
        // The strike should turn grass into an exposed ore crater (Dirt)
        assert_eq!(grid.get(5, 5), Some(TerrainType::Dirt));
    }

    #[test]
    fn test_kinetic_strike_misses_hits_magma() {
        let mut app = App::new();
        app.add_systems(Update, kinetic_strike_system);

        // Setup real dependencies
        app.world_mut().insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        app.world_mut().insert_resource(BuildingMap::default());
        app.add_event::<BuildingRemovedEvent>();
        app.add_event::<AddChronicleEvent>();

        app.world_mut().spawn((
            SubsurfaceResource { kind: ResourceType::Ore, depth: 50.0 },
            GridPosition { x: 5, y: 5 },
        ));

        app.world_mut().spawn((
            // Fuel represents Magma
            SubsurfaceResource { kind: ResourceType::Fuel, depth: 40.0 },
            GridPosition { x: 6, y: 5 }, // Adjacent
        ));

        app.add_event::<KineticStrikeEvent>();
        // Simulate a strike that hit 6,5 (the accuracy offset made it land there, event.target_x = 6)
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 6,
            target_y: 5,
            accuracy_offset: 1.0,
        });

        app.update();

        let grid = app.world().get_resource::<TerrainGrid>().unwrap();
        // Missing and hitting magma creates a volcano (MagmaRock)
        assert_eq!(grid.get(6, 5), Some(TerrainType::MagmaRock));
    }
}
"""

with open("src/layer1/physics/kinetic_strike.rs", "w") as f:
    f.write(content)
