use bevy::prelude::*;
use scale::layer1::needs::Needs;
use scale::layer1::pop::PopDied;
use scale::layer1::stress::TraumaTracker;
use scale::layer1::entities::Pop;

// Declare the glue systems we are going to write
// (in scale::layer1::integration)
use scale::layer1::integration::{
    famine_tracking_system,
    trauma_decay_system,
    trauma_tracker_death_system,
};

#[test]
fn test_trauma_tracker_death_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<TraumaTracker>();
    app.add_event::<PopDied>();
    app.add_systems(Update, trauma_tracker_death_system);

    let pop = app.world_mut().spawn(Pop).id();
    app.world_mut()
        .resource_mut::<Events<PopDied>>()
        .send(PopDied {
            entity: pop,
            name: "John".to_string(),
            reason: "Starvation".to_string(),
            tick: 0
        });

    app.update();

    let tracker = app.world().resource::<TraumaTracker>();
    assert_eq!(tracker.recent_deaths, 1);
}

#[test]
fn test_famine_tracking_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<TraumaTracker>();
    app.add_systems(Update, famine_tracking_system);

    // Spawn a starving pop
    app.world_mut().spawn((
        Pop,
        Needs {
            hunger: 0.0,
            rest: 1.0,
            leisure: 1.0,
            hygiene: 1.0,
        },
    ));

    app.update();

    let tracker = app.world().resource::<TraumaTracker>();
    assert_eq!(tracker.famine_ticks, 1);
}

#[test]
fn test_trauma_decay_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(TraumaTracker {
        recent_deaths: 10,
        famine_ticks: 10,
    });
    app.add_systems(Update, trauma_decay_system);

    // Default decay might be slow, so let's run it enough times to see a change
    for _ in 0..1500 {
        app.update();
    }

    let tracker = app.world().resource::<TraumaTracker>();
    assert!(tracker.recent_deaths < 10, "recent_deaths should decay");
    assert!(tracker.famine_ticks < 10, "famine_ticks should decay");
}
