use bevy_ecs::prelude::*;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer1::map::GridPosition;
use crate::layer1::building::{BuildingMap, OccupiedTiles};
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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
        // In this implementation, the event target coordinates already account for the offset
        let actual_hit_x = event.target_x;
        let actual_hit_y = event.target_y;

        // Destroy building if present
        let mut despawned_building = false;
        if let Some(ref mut map) = building_map {
            if let Some(&entity) = map.0.get(&(actual_hit_x, actual_hit_y)) {
                commands.entity(entity).despawn();
                map.0.remove(&(actual_hit_x, actual_hit_y));
                despawned_building = true;
                chronicle_events.send(AddChronicleEvent {
                    text: format!("A building was destroyed by a kinetic strike at {}, {}", actual_hit_x, actual_hit_y),
                    importance: EventImportance::Major,
            ..Default::default()});
            }
        }

        if despawned_building {
            if let Some(ref mut occ) = occupied {
                occ.0.remove(&(actual_hit_x, actual_hit_y));
            }
        }

        let mut kind_found = None;
        for (entity, resource, pos) in subsurface_query.iter() {
            if pos.x == actual_hit_x && pos.y == actual_hit_y {
                kind_found = Some(resource.kind);
                commands.entity(entity).despawn(); // Consume subsurface resource
                break; // Assuming one resource per tile
            }
        }

        if actual_hit_x < 0 || actual_hit_y < 0 {
            continue;
        }

        let tx = actual_hit_x as usize;
        let ty = actual_hit_y as usize;

        if tx < grid.width && ty < grid.height {
            match kind_found {
                Some(SubsurfaceResourceKind::OreVein) => {
                    grid.set(tx, ty, TerrainType::Crater);
                    chronicle_events.send(AddChronicleEvent {
                        text: format!("Kinetic strike exposed an Ore Vein at {}, {}", actual_hit_x, actual_hit_y),
                        importance: EventImportance::Standard,
            ..Default::default()});
                }
                Some(SubsurfaceResourceKind::Magma) => {
                    grid.set(tx, ty, TerrainType::MagmaRock);
                    chronicle_events.send(AddChronicleEvent {
                        text: format!("Kinetic strike hit a magma pocket, creating an active Volcano at {}, {}", actual_hit_x, actual_hit_y),
                        importance: EventImportance::Major,
            ..Default::default()});
                }
                None => {
                     // No subsurface resource, just make a crater
                     grid.set(tx, ty, TerrainType::Crater);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::nature::terrain::TerrainType;
    use crate::layer1::building::{Building, BuildingType};

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

        let _target_tile = app.world_mut().spawn((
            SubsurfaceResource { kind: SubsurfaceResourceKind::OreVein, depth: 50.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        app.add_event::<KineticStrikeEvent>();
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 5,
            target_y: 5,
            accuracy_offset: 0.0,
        });

        app.update();

        let terrain = app.world().resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5).unwrap(), TerrainType::Crater);
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

        let _target_tile = app.world_mut().spawn((
            SubsurfaceResource { kind: SubsurfaceResourceKind::OreVein, depth: 50.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        let _adjacent_tile = app.world_mut().spawn((
            SubsurfaceResource { kind: SubsurfaceResourceKind::Magma, depth: 40.0 },
            GridPosition { x: 6, y: 5 },
        )).id();

        app.add_event::<KineticStrikeEvent>();
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 6,
            target_y: 5,
            accuracy_offset: 1.0,
        });

        app.update();

        let terrain = app.world().resource::<TerrainGrid>();
        assert_eq!(terrain.get(6, 5).unwrap(), TerrainType::MagmaRock);
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
        occupied.0.insert((5, 5));
        app.world_mut().insert_resource(occupied);

        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 5, y: 5 },
        )).id();

        let mut map = BuildingMap::default();
        map.0.insert((5, 5), building);
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
        assert_eq!(terrain.get(5, 5).unwrap(), TerrainType::Crater);
    }
}
