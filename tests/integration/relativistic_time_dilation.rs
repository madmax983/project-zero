use bevy::prelude::*;
use scale::layer2::fleet::{Fleet, InOrbit};
use scale::layer3::physics::relativity::{
    process_time_dilation_system, update_fleet_local_time_system, LocalTimeTracker, SimulationTime,
    StationedAt, SystemNode, TimeDilationZone,
};
use scale::layer2::integration::in_orbit_to_stationed_at_bridge_system;

#[test]
fn test_in_orbit_grants_stationed_at_and_receives_time_dilation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SimulationTime>();

    // Add the integration system AND the relativity systems
    app.add_systems(
        Update,
        (
            in_orbit_to_stationed_at_bridge_system,
            process_time_dilation_system,
            update_fleet_local_time_system,
        )
            .chain(),
    );

    // Spawn a system node with time dilation (1 local tick per 5 global ticks)
    let system_node = app
        .world_mut()
        .spawn((
            SystemNode,
            TimeDilationZone { dilation_factor: 5 },
            LocalTimeTracker::default(),
        ))
        .id();

    // Spawn a fleet in orbit around the system node
    let fleet = app
        .world_mut()
        .spawn((
            Fleet,
            InOrbit {
                parent: system_node,
            },
            LocalTimeTracker::default(),
        ))
        .id();

    // Initialize StationedAt component
    app.update();

    // Advance 5 global ticks
    for _ in 0..5 {
        app.world_mut().resource_mut::<SimulationTime>().tick += 1;
        app.update();
    }

    // Verify the fleet gained the StationedAt component
    assert!(
        app.world().get::<StationedAt>(fleet).is_some(),
        "Fleet should have gained StationedAt via the bridge system"
    );

    // Verify the fleet's StationedAt points to the system node
    let stationed_at = app.world().get::<StationedAt>(fleet).unwrap();
    assert_eq!(stationed_at.0, system_node);

    let sys_tracker = app.world().get::<LocalTimeTracker>(system_node).unwrap();
    assert_eq!(sys_tracker.local_ticks, 1, "System node tracker failed to update");

    let fleet_tracker = app.world().get::<LocalTimeTracker>(fleet).unwrap();
    assert_eq!(fleet_tracker.local_ticks, 1, "Fleet tracker failed to update");
}

#[test]
fn test_leaving_orbit_removes_stationed_at() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, in_orbit_to_stationed_at_bridge_system);

    let system_node = app.world_mut().spawn_empty().id();

    // Spawn a fleet in orbit
    let fleet = app
        .world_mut()
        .spawn((
            Fleet,
            InOrbit {
                parent: system_node,
            },
        ))
        .id();

    // Run the system to add StationedAt
    app.update();

    assert!(app.world().get::<StationedAt>(fleet).is_some());

    // Remove InOrbit
    app.world_mut().entity_mut(fleet).remove::<InOrbit>();

    // Run the system to remove StationedAt
    app.update();

    assert!(
        app.world().get::<StationedAt>(fleet).is_none(),
        "StationedAt should be removed when InOrbit is removed"
    );
}
