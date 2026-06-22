use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer3::integration::zoo_hypothesis_chronicle_bridge;
use scale::layer3::zoo_hypothesis::{
    evaluate_colony_entertainment_system, trigger_alien_reward_system, AlienObservers,
};
use scale::layer1::environment::disasters::DisasterEvent;

#[test]
fn test_zoo_hypothesis_chronicle_bridge() {
    let mut app = bevy_app::App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.insert_resource(AlienObservers {
        entertainment_score: 150.0,
        threshold: 100.0,
    });

    app.init_resource::<Events<DisasterEvent>>();
    app.init_resource::<Events<scale::layer1::crafting::CraftEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(
        bevy_app::Update,
        (
            evaluate_colony_entertainment_system,
            trigger_alien_reward_system,
            zoo_hypothesis_chronicle_bridge,
        )
            .chain(),
    );

    // Initial state: no chronicle events
    app.world_mut().resource_mut::<Events<AddChronicleEvent>>().clear();

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let chronicle_events: Vec<_> = reader.read(events).collect();

    assert_eq!(chronicle_events.len(), 1, "Expected one chronicle event when drop pod spawns.");
    assert!(chronicle_events[0].text.contains("An anomalous supply drop has fallen from orbit"));
}
