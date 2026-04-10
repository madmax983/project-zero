use bevy::prelude::*;
use scale::layer2::fleet::{Fleet, InOrbit};
use scale::layer3::map::StarSystem;
use scale::layer3::physics::relativity::{
    process_time_dilation_system, update_fleet_local_time_system, LocalTimeTracker,
    TimeDilationZone,
};
use scale::shared::time::SimulationTime;

#[test]
fn test_time_dilation_bridge_updates_fleet() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(scale::shared::time::SimulationTime::default());

    // Register systems from both modules
    app.add_systems(
        Update,
        (
            process_time_dilation_system,
            update_fleet_local_time_system,
        )
            .chain(),
    );

    // Setup: a star system with time dilation
    let system_entity = app
        .world_mut()
        .spawn((
            StarSystem { id: 1 },
            TimeDilationZone { dilation_factor: 5 }, // 1 local tick per 5 global ticks
            LocalTimeTracker::default(),
        ))
        .id();

    // Setup: a fleet stationed at the star system
    let fleet_entity = app
        .world_mut()
        .spawn((
            Fleet,
            InOrbit {
                parent: system_entity,
            },
            LocalTimeTracker::default(),
        ))
        .id();

    // Act: run 5 update cycles (which is 1 local tick)
    for _ in 0..5 {
        app.world_mut().resource_mut::<SimulationTime>().tick += 1;
        app.update();
    }

    // Assert: fleet's local time tracker was updated (time flowed across the seam)
    let fleet_tracker = app.world().get::<LocalTimeTracker>(fleet_entity).unwrap();
    assert_eq!(
        fleet_tracker.local_ticks, 1,
        "Fleet should have 1 local tick after 5 global ticks due to dilation factor 5"
    );
}
