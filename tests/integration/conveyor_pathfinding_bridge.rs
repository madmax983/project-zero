use bevy::prelude::*;
use scale::layer1::architecture::building::{
    Building, BuildingMap, BuildingType, Direction, OccupiedTiles,
};
use scale::layer1::core::map::GridPosition;
use scale::layer1::logistics::conveyor::{BeltVariant, ConveyorBelt};
use scale::layer1::pathfinding::find_path;
use scale::layer1::terrain::{TerrainGrid, TerrainType};

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![TerrainType::Grass; 100],
    });
    app.insert_resource(OccupiedTiles::default());
    app.insert_resource(BuildingMap::default());
    app.add_systems(
        Update,
        scale::layer1::architecture::building::update_building_map_system,
    );
    app
}

#[test]
fn test_standard_conveyor_blocks_pathfinding() {
    let mut app = setup_app();

    // Block entire column x=1 except y=1
    for y in 0..10 {
        if y == 1 {
            continue;
        }
        app.world_mut().spawn((
            GridPosition { x: 1, y },
            Building {
                building_type: BuildingType::Wall,
            },
        ));
        app.world_mut()
            .resource_mut::<OccupiedTiles>()
            .0
            .insert((1, y));
    }
    app.world_mut().spawn((
        GridPosition { x: 1, y: 1 },
        Building {
            building_type: BuildingType::ConveyorBelt,
        },
        ConveyorBelt {
            direction: Direction::East,
            speed: 1.0,
            variant: BeltVariant::Standard,
        },
    ));

    app.world_mut()
        .resource_mut::<OccupiedTiles>()
        .0
        .insert((1, 1));
    app.update();

    let path = find_path(app.world(), (0, 1), (2, 1));
    assert!(path.is_none(), "Standard conveyor must block pathfinding");
}

#[test]
fn test_underground_conveyor_allows_pathfinding() {
    let mut app = setup_app();

    // Block entire column x=1 except y=1
    for y in 0..10 {
        if y == 1 {
            continue;
        }
        app.world_mut().spawn((
            GridPosition { x: 1, y },
            Building {
                building_type: BuildingType::Wall,
            },
        ));
        app.world_mut()
            .resource_mut::<OccupiedTiles>()
            .0
            .insert((1, y));
    }
    app.world_mut().spawn((
        GridPosition { x: 1, y: 1 },
        Building {
            building_type: BuildingType::ConveyorBelt,
        },
        ConveyorBelt {
            direction: Direction::East,
            speed: 1.0,
            variant: BeltVariant::Underground,
        },
    ));

    app.world_mut()
        .resource_mut::<OccupiedTiles>()
        .0
        .insert((1, 1));
    app.update();

    let path = find_path(app.world(), (0, 1), (2, 1));
    assert!(
        path.is_some(),
        "Underground conveyor must allow pathfinding"
    );
}
