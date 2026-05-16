use bevy::prelude::*;
use scale::layer1::administration::invasive_bureaucracy::{
    calculate_bureaucracy_stability_system, expand_bureaucracy_nodes_system, BureaucracyNode,
    EmpireStability,
};
use scale::layer1::building::{Building, BuildingMap, BuildingType, OccupiedTiles};
use scale::layer1::events::BuildingRemovedEvent;
use scale::layer1::map::GridPosition;
use scale::layer1::terrain::{TerrainGrid, TerrainType};

#[test]
fn test_bureaucracy_expansion_to_empire_stability_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Minimal resources needed
    app.insert_resource(TerrainGrid {
        width: 5,
        height: 5,
        tiles: vec![TerrainType::Grass; 25],
    });
    app.init_resource::<OccupiedTiles>();
    app.init_resource::<BuildingMap>();
    app.init_resource::<EmpireStability>();
    app.add_event::<BuildingRemovedEvent>();

    // Register systems
    app.add_systems(
        Update,
        (
            expand_bureaucracy_nodes_system,
            calculate_bureaucracy_stability_system,
        )
            .chain(),
    );

    // Initial stability should be 0
    app.update();
    assert_eq!(app.world().resource::<EmpireStability>().value, 0.0);

    // Spawn the first bureaucracy node
    let node_entity = app
        .world_mut()
        .spawn((
            BureaucracyNode {
                expansion_timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
            GridPosition { x: 2, y: 2 },
            Building {
                building_type: BuildingType::Office,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<OccupiedTiles>()
        .0
        .insert((2, 2));
    app.world_mut()
        .resource_mut::<BuildingMap>()
        .0
        .insert((2, 2), node_entity);

    // After adding the node but before it expands, stability should increase due to the calculation system
    app.update();
    assert_eq!(
        app.world().resource::<EmpireStability>().value,
        10.0,
        "One node should provide 10.0 stability"
    );

    // Verify there is exactly 1 node
    let mut query = app.world_mut().query::<&BureaucracyNode>();
    assert_eq!(query.iter(app.world()).count(), 1);

    // Fast-forward time to trigger expansion
    for _ in 0..10 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(2));
        app.update();

        let node_count = app
            .world_mut()
            .query::<&BureaucracyNode>()
            .iter(app.world())
            .count();
        if node_count > 1 {
            break;
        }

        // Timer reset for next iter
        let mut query2 = app.world_mut().query::<&mut BureaucracyNode>();
        for mut node in query2.iter_mut(app.world_mut()) {
            node.expansion_timer
                .set_elapsed(std::time::Duration::from_secs(2));
        }
    }

    // Now there should be more nodes, and stability should be higher
    let node_count = app
        .world_mut()
        .query::<&BureaucracyNode>()
        .iter(app.world())
        .count();
    assert!(node_count > 1, "The node should have expanded");

    let expected_stability = node_count as f32 * 10.0;
    assert_eq!(
        app.world().resource::<EmpireStability>().value,
        expected_stability,
        "Stability should be recalculated after expansion"
    );
}
