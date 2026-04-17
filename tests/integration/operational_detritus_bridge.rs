use bevy::prelude::*;
use scale::layer1::beauty::update_beauty_grid_system;
use scale::layer1::clutter::{clutter_accumulation_system, clutter_cleaning_system, ClutterGrid};
use scale::layer1::pathfinding::find_path;
use scale::layer1::pop::Pop;
use scale::layer1::utility_types::{ActionType, PopAction};
use scale::layer1::GridPosition;

#[test]
fn test_operational_detritus_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add resources
    app.world_mut().insert_resource(ClutterGrid::new(10, 10));
    app.world_mut().insert_resource(scale::layer1::beauty::BeautyGrid::new(10, 10));
    app.world_mut().insert_resource(scale::layer1::TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![scale::layer1::terrain::TerrainType::Grass; 100],
    });
    app.world_mut().insert_resource(scale::layer1::building::OccupiedTiles::default());
    app.world_mut().insert_resource(scale::layer1::building::BuildingMap::default());
    app.world_mut().insert_resource(scale::layer1::crowding::CrowdingGrid::new(10, 10));

    // Add systems
    app.add_systems(
        Update,
        (
            clutter_accumulation_system,
            clutter_cleaning_system,
            update_beauty_grid_system,
        )
            .chain(),
    );

    // Setup a pop working at 5,5
    let _worker = app.world_mut().spawn((
        Pop,
        GridPosition { x: 5, y: 5 },
        PopAction {
            current: ActionType::Work,
            ..Default::default()
        },
    )).id();

    // Run simulation to accumulate clutter
    app.update();

    // Check clutter accumulated
    let clutter = app.world().resource::<ClutterGrid>().get(5, 5);
    assert!(clutter > 0.0, "Clutter should accumulate from work");

    // Check beauty impacted
    let beauty = app.world().resource::<scale::layer1::beauty::BeautyGrid>().get(5, 5);
    assert!(beauty < scale::layer1::beauty::GRASS_BEAUTY, "Beauty should be penalized by clutter, expected less than {}, got {}", scale::layer1::beauty::GRASS_BEAUTY, beauty);

    // Check pathfinding
    app.world_mut().resource_mut::<ClutterGrid>().set(1, 0, 100.0);
    let path = find_path(app.world(), (0, 0), (2, 0));
    if let Some(p) = path {
        assert!(!p.contains(&(1, 0)), "Path should avoid high clutter");
    }

    // Spawn janitor
    let _janitor = app.world_mut().spawn((
        Pop,
        GridPosition { x: 5, y: 5 },
        PopAction {
            current: ActionType::Clean,
            ..Default::default()
        },
    )).id();

    // Run simulation to clean
    app.update();

    let clutter_after = app.world().resource::<ClutterGrid>().get(5, 5);
    assert!(clutter_after < clutter, "Janitor should clean clutter");
}
