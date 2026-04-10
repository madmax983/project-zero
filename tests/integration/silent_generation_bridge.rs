use bevy::prelude::*;
use scale::layer1::integration::{
    famine_tracking_system, trauma_death_bridge_system, trauma_decay_system,
};
use scale::layer1::pop::PopDied;
use scale::layer1::resources::ColonyResources;
use scale::layer1::stress::TraumaTracker;
use scale::shared::time::SimulationTime;

#[test]
fn test_trauma_death_bridge_increments_recent_deaths() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<TraumaTracker>();
    app.add_event::<PopDied>();

    app.add_systems(Update, trauma_death_bridge_system);

    // Send a PopDied event
    app.world_mut().send_event(PopDied {
        entity: Entity::PLACEHOLDER,
        tick: 1,
        name: "Test Pop".to_string(),
        reason: "Starvation".to_string(),
    });

    app.update();

    let trauma = app.world().get_resource::<TraumaTracker>().unwrap();
    assert_eq!(
        trauma.recent_deaths, 1,
        "recent_deaths should increment on PopDied"
    );
}

#[test]
fn test_famine_tracking_increments_famine_ticks_when_food_low() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<TraumaTracker>();
    app.insert_resource(ColonyResources {
        food: 0.0, // Low food
        ..Default::default()
    });

    app.add_systems(Update, famine_tracking_system);

    app.update();

    let trauma = app.world().get_resource::<TraumaTracker>().unwrap();
    assert_eq!(
        trauma.famine_ticks, 1,
        "famine_ticks should increment when food is low"
    );

    // Add food, should not increment
    app.world_mut().resource_mut::<ColonyResources>().food = 100.0;
    app.update();

    let trauma = app.world().get_resource::<TraumaTracker>().unwrap();
    assert_eq!(
        trauma.famine_ticks, 1,
        "famine_ticks should not increment when food is available"
    );
}

#[test]
fn test_trauma_decay_system_decays_values() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(TraumaTracker {
        recent_deaths: 10,
        famine_ticks: 20,
    });
    app.insert_resource(SimulationTime {
        tick: 10,
        speed: scale::shared::time::SimSpeed::Normal,
    });

    app.add_systems(Update, trauma_decay_system);

    app.update();

    let trauma = app.world().get_resource::<TraumaTracker>().unwrap();
    assert_eq!(
        trauma.recent_deaths, 9,
        "recent_deaths should decay every 10 ticks"
    );
    assert_eq!(
        trauma.famine_ticks, 19,
        "famine_ticks should decay every 10 ticks"
    );

    // Next tick is 11, shouldn't decay
    app.world_mut().resource_mut::<SimulationTime>().tick = 11;
    app.update();

    let trauma = app.world().get_resource::<TraumaTracker>().unwrap();
    assert_eq!(
        trauma.recent_deaths, 9,
        "recent_deaths should only decay on multiples of 10 ticks"
    );
    assert_eq!(
        trauma.famine_ticks, 19,
        "famine_ticks should only decay on multiples of 10 ticks"
    );
}
