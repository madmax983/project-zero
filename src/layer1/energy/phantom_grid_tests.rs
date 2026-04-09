use crate::layer1::energy::phantom_grid::{
    phantom_grid_connection_system, phantom_grid_disconnection_system, phantom_grid_hum_system,
    PhantomGridCapable, PhantomGridConnected,
};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use crate::layer1::stress::StressTracker;
use bevy::prelude::*;

#[test]
fn test_unpowered_building_taps_phantom_grid() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, phantom_grid_connection_system);

    // Create a building that requires power but has none
    let building = app
        .world_mut()
        .spawn((
            PowerConsumer {
                demand: 10.0,
                active: false,
            },
            PhantomGridCapable {
                connection_chance: 1.0, // 100% chance for testing
            },
        ))
        .id();

    // Act
    app.update();

    // Assert: The building should now be connected and powered by the phantom grid
    let consumer = app.world().get::<PowerConsumer>(building).unwrap();
    assert!(consumer.active);
    assert!(app.world().get::<PhantomGridConnected>(building).is_some());
}

#[test]
fn test_phantom_grid_emits_stress_hum() {
    // Arrange
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.world_mut()
        .resource_mut::<Time<()>>()
        .advance_by(std::time::Duration::from_secs(1));

    app.add_systems(Update, phantom_grid_hum_system);

    // Create a connected building
    app.world_mut()
        .spawn((GridPosition { x: 0, y: 0 }, PhantomGridConnected));

    // Create a nearby pop
    let pop = app
        .world_mut()
        .spawn((
            GridPosition { x: 1, y: 0 }, // Distance 1
            StressTracker::default(),
        ))
        .id();

    // Act
    app.update();

    // Assert: The pop's stress should have increased due to the hum
    let tracker = app.world().get::<StressTracker>(pop).unwrap();
    assert!(tracker.accumulated_stress > 0.0);
}

#[test]
fn test_building_disconnects_when_normal_power_restored() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, phantom_grid_disconnection_system);

    // Create a building that is connected to the phantom grid, but now has its normal power restored
    let building = app
        .world_mut()
        .spawn((
            PowerConsumer {
                demand: 10.0,
                active: true,
            },
            PhantomGridConnected,
        ))
        .id();

    // Act
    app.update();

    // Assert: The building should disconnect from the phantom grid
    assert!(app.world().get::<PhantomGridConnected>(building).is_none());
}
