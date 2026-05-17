use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::map::GridPosition;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer1::{BuildingMap, OccupiedTiles};
use bevy_ecs::prelude::*;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SubsurfaceResourceKind {
    OreVein,
    Magma,
}

#[derive(Component)]
pub struct SubsurfaceResource {
    pub kind: SubsurfaceResourceKind,
    pub depth: f32,
}

#[derive(Event)]
pub struct KineticStrikeEvent {
    pub target_x: i32,
    pub target_y: i32,
    pub accuracy_offset: f32,
}

pub fn kinetic_strike_system(
    mut events: EventReader<KineticStrikeEvent>,
    mut grid: ResMut<TerrainGrid>,
    subsurface_query: Query<(Entity, &SubsurfaceResource, &GridPosition)>,
    mut commands: Commands,
    mut occupied: Option<ResMut<OccupiedTiles>>,
    mut building_map: Option<ResMut<BuildingMap>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        let actual_hit_x = event.target_x;
        let actual_hit_y = event.target_y;

        chronicle_events.send(AddChronicleEvent {
            text: "The Colony Called Down the Thunder".to_string(),
            importance: EventImportance::Major,
        });

        let mut kind_found = None;
        for (entity, resource, pos) in subsurface_query.iter() {
            if pos.x == actual_hit_x && pos.y == actual_hit_y {
                kind_found = Some(resource.kind);
                commands.entity(entity).despawn();
                break;
            }
        }

        for y in (actual_hit_y - 1)..=(actual_hit_y + 1) {
            for x in (actual_hit_x - 1)..=(actual_hit_x + 1) {
                let mut despawned_building = false;
                if let Some(ref mut map) = building_map {
                    if let Some(&entity) = map.0.get(&(x, y)) {
                        commands.entity(entity).despawn();
                        map.0.remove(&(x, y));
                        despawned_building = true;
                        chronicle_events.send(AddChronicleEvent {
                            text: format!(
                                "A building was destroyed by a kinetic strike at {}, {}",
                                x, y
                            ),
                            importance: EventImportance::Major,
                        });
                    }
                }

                if despawned_building {
                    if let Some(ref mut occ) = occupied {
                        occ.0.remove(&(x, y));
                    }
                }

                if x >= 0 && y >= 0 {
                    let tx = x as usize;
                    let ty = y as usize;
                    if tx < grid.width && ty < grid.height {
                        let new_terrain = match kind_found {
                            Some(SubsurfaceResourceKind::Magma) => TerrainType::MagmaRock,
                            _ => TerrainType::Crater,
                        };
                        grid.set(tx, ty, new_terrain);
                    }
                }
            }
        }

        for (entity, _resource, pos) in subsurface_query.iter() {
            let in_x_range = pos.x >= actual_hit_x - 1 && pos.x <= actual_hit_x + 1;
            let in_y_range = pos.y >= actual_hit_y - 1 && pos.y <= actual_hit_y + 1;
            if in_x_range && in_y_range && (pos.x != actual_hit_x || pos.y != actual_hit_y) {
                commands.entity(entity).despawn();
            }
        }

        match kind_found {
            Some(SubsurfaceResourceKind::OreVein) => {
                chronicle_events.send(AddChronicleEvent {
                    text: format!(
                        "Kinetic strike exposed an Ore Vein at {}, {}",
                        actual_hit_x, actual_hit_y
                    ),
                    importance: EventImportance::Standard,
                });
            }
            Some(SubsurfaceResourceKind::Magma) => {
                chronicle_events.send(AddChronicleEvent {
                    text: format!(
                        "Kinetic strike hit a magma pocket, creating an active Volcano at {}, {}",
                        actual_hit_x, actual_hit_y
                    ),
                    importance: EventImportance::Major,
                });
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::nature::terrain::TerrainType;
    use bevy::prelude::*;

    #[test]
    fn test_kinetic_strike_exposes_ore() {
        let mut app = App::new();
        app.add_systems(Update, kinetic_strike_system);

        let grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        app.world_mut().insert_resource(grid);
        app.world_mut().init_resource::<Events<AddChronicleEvent>>();

        let _target_tile = app
            .world_mut()
            .spawn((
                SubsurfaceResource {
                    kind: SubsurfaceResourceKind::OreVein,
                    depth: 50.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        app.add_event::<KineticStrikeEvent>();
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 5,
            target_y: 5,
            accuracy_offset: 0.0,
        });

        app.update();

        let terrain = app.world().resource::<TerrainGrid>();
        assert_eq!(terrain.get(4, 4).unwrap(), TerrainType::Crater);
        assert_eq!(terrain.get(5, 5).unwrap(), TerrainType::Crater);
        assert_eq!(terrain.get(6, 6).unwrap(), TerrainType::Crater);
        assert_eq!(terrain.get(7, 7).unwrap(), TerrainType::Grass);
    }

    #[test]
    fn test_kinetic_strike_misses_hits_magma() {
        let mut app = App::new();
        app.add_systems(Update, kinetic_strike_system);

        let grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        app.world_mut().insert_resource(grid);
        app.world_mut().init_resource::<Events<AddChronicleEvent>>();

        let _target_tile = app
            .world_mut()
            .spawn((
                SubsurfaceResource {
                    kind: SubsurfaceResourceKind::OreVein,
                    depth: 50.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let _adjacent_tile = app
            .world_mut()
            .spawn((
                SubsurfaceResource {
                    kind: SubsurfaceResourceKind::Magma,
                    depth: 40.0,
                },
                GridPosition { x: 6, y: 5 },
            ))
            .id();

        app.add_event::<KineticStrikeEvent>();
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 6,
            target_y: 5,
            accuracy_offset: 1.0,
        });

        app.update();

        let terrain = app.world().resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 4).unwrap(), TerrainType::MagmaRock);
        assert_eq!(terrain.get(6, 5).unwrap(), TerrainType::MagmaRock);
        assert_eq!(terrain.get(7, 6).unwrap(), TerrainType::MagmaRock);
        assert_eq!(terrain.get(8, 7).unwrap(), TerrainType::Grass);
    }

    #[test]
    fn test_kinetic_strike_destroys_surface_building() {
        let mut app = App::new();
        app.add_systems(Update, kinetic_strike_system);

        let grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        app.world_mut().insert_resource(grid);
        app.world_mut().init_resource::<Events<AddChronicleEvent>>();

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((4, 5));
        app.world_mut().insert_resource(occupied);

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 4, y: 5 },
            ))
            .id();

        let mut map = BuildingMap::default();
        map.0.insert((4, 5), building);
        app.world_mut().insert_resource(map);

        app.add_event::<KineticStrikeEvent>();
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 5,
            target_y: 5,
            accuracy_offset: 0.0,
        });

        app.update();

        assert!(app.world().get_entity(building).is_err());
        let terrain = app.world().resource::<TerrainGrid>();
        assert_eq!(terrain.get(4, 5).unwrap(), TerrainType::Crater);
        assert_eq!(terrain.get(5, 5).unwrap(), TerrainType::Crater);
    }

    #[test]
    fn test_kinetic_strike_oob() {
        let mut app = App::new();
        app.add_systems(Update, kinetic_strike_system);

        let grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        app.world_mut().insert_resource(grid);
        app.world_mut().init_resource::<Events<AddChronicleEvent>>();

        app.add_event::<KineticStrikeEvent>();
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 0,
            target_y: 0,
            accuracy_offset: 0.0,
        });

        app.update();

        let terrain = app.world().resource::<TerrainGrid>();
        assert_eq!(terrain.get(0, 0).unwrap(), TerrainType::Crater);
        assert_eq!(terrain.get(1, 0).unwrap(), TerrainType::Crater);
        assert_eq!(terrain.get(0, 1).unwrap(), TerrainType::Crater);
        assert_eq!(terrain.get(1, 1).unwrap(), TerrainType::Crater);
        assert_eq!(terrain.get(2, 2).unwrap(), TerrainType::Grass);
    }
}
