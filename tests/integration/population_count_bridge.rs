use bevy::prelude::*;
use scale::layer1::pop::{PopBorn, PopDied, PopulationCount};
use scale::layer1::core::integration::{pop_born_count_system, pop_died_count_system};

#[test]
fn test_population_count_updates() {
    let mut app = App::new();

    // 1. Init Resources
    app.insert_resource(PopulationCount { total: 10 });
    app.add_event::<PopBorn>();
    app.add_event::<PopDied>();

    // 2. Add systems
    app.add_systems(Update, (pop_born_count_system, pop_died_count_system));

    // 3. Test PopBorn
    app.world_mut().send_event(PopBorn {
        entity: Entity::from_raw(1),
        name: "Newborn".to_string(),
        tick: 0,
        source: "Test".to_string(),
    });

    app.update();

    assert_eq!(
        app.world().resource::<PopulationCount>().total,
        11,
        "PopulationCount should increase when PopBorn is sent"
    );

    // 4. Test PopDied
    app.world_mut().send_event(PopDied {
        entity: Entity::from_raw(2),
        name: "Deadman".to_string(),
        tick: 0,
        reason: "Old age".to_string(),
    });

    app.update();

    assert_eq!(
        app.world().resource::<PopulationCount>().total,
        10,
        "PopulationCount should decrease when PopDied is sent"
    );
}
