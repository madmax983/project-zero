use bevy::prelude::*;
use scale::layer1::pop::PopDied;
use scale::layer1::stress::TraumaTracker;
use scale::layer1::integration::{update_trauma_tracker_deaths_system, update_trauma_tracker_famine_system, decay_trauma_tracker_system};
use scale::layer1::economy::resources::ColonyResources;

#[test]
fn test_pop_died_increments_recent_deaths() {
    let mut app = App::new();
    app.init_resource::<TraumaTracker>();
    app.add_event::<PopDied>();
    app.add_systems(Update, update_trauma_tracker_deaths_system);

    app.world_mut().send_event(PopDied { entity: Entity::PLACEHOLDER, reason: "starvation".to_string(), name: "Bob".to_string(), tick: 0 });
    app.update();

    let tracker = app.world().resource::<TraumaTracker>();
    assert_eq!(tracker.recent_deaths, 1);
}

#[test]
fn test_famine_increments_famine_ticks() {
    let mut app = App::new();
    app.init_resource::<TraumaTracker>();
    let resources = ColonyResources::zeroed();
    // No food
    app.insert_resource(resources);
    app.add_systems(Update, update_trauma_tracker_famine_system);

    app.update();

    let tracker = app.world().resource::<TraumaTracker>();
    assert_eq!(tracker.famine_ticks, 1);
}

#[test]
fn test_decay_trauma_tracker() {
    let mut app = App::new();
    app.insert_resource(TraumaTracker {
        recent_deaths: 10,
        famine_ticks: 5,
    });
    use scale::shared::time::SimulationTime;
    app.insert_resource(SimulationTime { tick: 60, ..Default::default() });

    // Give food so famine ticks don't increment, we just test decay
    let mut resources = ColonyResources::zeroed();
    resources.food = 100.0;
    app.insert_resource(resources);

    app.add_systems(Update, decay_trauma_tracker_system);

    app.update();

    let tracker = app.world().resource::<TraumaTracker>();
    assert!(tracker.recent_deaths < 10, "recent_deaths was {}", tracker.recent_deaths);
    assert!(tracker.famine_ticks < 5);
}
