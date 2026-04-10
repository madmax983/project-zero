use bevy::prelude::*;
use scale::layer1::integration::{
    famine_tracking_system, trauma_death_bridge_system, trauma_decay_system,
};
use scale::layer1::pop::PopDied;
use scale::layer1::resources::ColonyResources;
use scale::layer1::stress::TraumaTracker;
use scale::shared::time::{SimSpeed, SimulationTime};

#[test]
fn test_trauma_death_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PopDied>();
    app.init_resource::<TraumaTracker>();

    app.add_systems(Update, trauma_death_bridge_system);

    app.world_mut().send_event(PopDied {
        entity: Entity::PLACEHOLDER,
        name: "test".to_string(),
        reason: "old age".to_string(),
        tick: 0,
    });
    app.world_mut().send_event(PopDied {
        entity: Entity::PLACEHOLDER,
        name: "test".to_string(),
        reason: "old age".to_string(),
        tick: 0,
    });

    app.update();

    let trauma = app.world().resource::<TraumaTracker>();
    assert_eq!(trauma.recent_deaths, 2);
}

#[test]
fn test_famine_tracking_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<TraumaTracker>();
    app.insert_resource(ColonyResources {
        food: 0.0,
        ..Default::default()
    });

    app.add_systems(Update, famine_tracking_system);

    app.update();
    app.update();

    let trauma = app.world().resource::<TraumaTracker>();
    assert_eq!(trauma.famine_ticks, 2);

    app.world_mut().resource_mut::<ColonyResources>().food = 10.0;
    app.update();

    let trauma2 = app.world().resource::<TraumaTracker>();
    assert_eq!(trauma2.famine_ticks, 2, "Should not increase when food > 0");
}

#[test]
fn test_trauma_decay_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TraumaTracker {
        recent_deaths: 10,
        famine_ticks: 100,
    });
    app.insert_resource(SimulationTime {
        tick: 10, // Assuming decay happens every 10 ticks
        speed: SimSpeed::Normal,
    });

    app.add_systems(Update, trauma_decay_system);

    app.update();

    let trauma = app.world().resource::<TraumaTracker>();
    assert_eq!(trauma.recent_deaths, 9);
    assert_eq!(trauma.famine_ticks, 90);
}
