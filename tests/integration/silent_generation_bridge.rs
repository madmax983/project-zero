use bevy::prelude::*;
use scale::layer1::entities::pop::PopDied;
use scale::layer1::integration::silent_generation_trauma_bridge_system;
use scale::layer1::resources::ColonyResources;
use scale::layer1::stress::TraumaTracker;

#[test]
fn test_silent_generation_trauma_bridge() {
    let mut app = App::new();

    // 1. Setup Resources
    app.insert_resource(ColonyResources {
        food: 0.0,
        ..Default::default()
    });
    app.insert_resource(TraumaTracker {
        recent_deaths: 0,
        famine_ticks: 0,
    });

    // 2. Setup Events
    app.add_event::<PopDied>();

    // 3. Setup Systems
    app.add_systems(Update, silent_generation_trauma_bridge_system);

    // 4. Send Event
    app.world_mut().send_event(PopDied {
        entity: Entity::from_raw(1),
        name: "Test Pop".to_string(),
        reason: "Test".to_string(),
        tick: 0,
    });

    // 5. Update
    app.update();

    // 6. Assert
    let trauma = app.world().resource::<TraumaTracker>();
    assert_eq!(trauma.recent_deaths, 1, "recent_deaths should be incremented");
    assert_eq!(trauma.famine_ticks, 1, "famine_ticks should be incremented because food is 0");

    // 7. Change food to positive
    app.world_mut().resource_mut::<ColonyResources>().food = 10.0;

    // 8. Update again
    app.update();

    // 9. Assert
    let trauma_2 = app.world().resource::<TraumaTracker>();
    assert_eq!(trauma_2.recent_deaths, 1, "recent_deaths should not increment if no deaths occurred");
    assert_eq!(trauma_2.famine_ticks, 0, "famine_ticks should reset/decay if food > 0");
}
