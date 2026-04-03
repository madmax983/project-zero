use bevy::prelude::*;
use scale::layer1::logistics::commute::{CommuteAction, CurrentLocation, process_orbital_commutes};
use scale::layer1::economy::ColonyResources;
use scale::layer1::entities::Pop;
use scale::layer3::physics::relativity::SystemNode;

#[test]
fn test_orbital_commute_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<ColonyResources>();
    app.world_mut().resource_mut::<ColonyResources>().fuel = 10.0;

    // In a real scenario, this would be registered in the execution schedule.
    // For this bridge test, we just ensure that chaining it directly works as expected.
    app.add_systems(Update, process_orbital_commutes);

    let planet_entity = app.world_mut().spawn(SystemNode).id();
    let station_entity = app.world_mut().spawn(SystemNode).id();

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            CurrentLocation(planet_entity),
            CommuteAction {
                destination: station_entity,
                fuel_cost: 2,
                duration: 3,
                progress: 0,
            },
        ))
        .id();

    // Tick 1
    app.update();
    let commute = app.world().get::<CommuteAction>(pop).unwrap();
    assert_eq!(commute.progress, 1);
    assert_eq!(app.world().resource::<ColonyResources>().fuel, 8.0);
    assert_eq!(
        app.world().get::<CurrentLocation>(pop).unwrap().0,
        planet_entity
    ); // Still en route

    // Tick 2
    app.update();

    // Tick 3 - Commute complete
    app.update();

    let loc = app.world().get::<CurrentLocation>(pop).unwrap();
    assert_eq!(loc.0, station_entity);
    assert!(app.world().get::<CommuteAction>(pop).is_none()); // Action removed
}
