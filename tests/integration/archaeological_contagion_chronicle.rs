use bevy::prelude::*;
use scale::layer1::archaeological_contagion::archaeological_infection_system;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::archaeological_contagion_chronicle_bridge;
use scale::layer1::deep_crust_resonance::ExcavationEvent;
use scale::layer1::entities::pop::Pop;

#[test]
fn test_archaeological_contagion_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<ExcavationEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (
            archaeological_infection_system,
            archaeological_contagion_chronicle_bridge,
        )
            .chain(),
    );

    let miner = app.world_mut().spawn(Pop).id();

    app.world_mut()
        .resource_mut::<Events<ExcavationEvent>>()
        .send(ExcavationEvent {
            colony: Entity::PLACEHOLDER,
            miner,
            discovery_type: "AncientRuins".to_string(),
            target: Entity::PLACEHOLDER,
        });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut iter = chronicle_events.get_cursor();
    let events: Vec<_> = iter.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Expected exactly 1 chronicle event for Ancient Ruins excavation"
    );
    assert!(events[0].text.contains("Ancient Routine"));
}
