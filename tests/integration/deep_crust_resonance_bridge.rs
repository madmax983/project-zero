use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::deep_crust_resonance_chronicle_bridge;
use scale::layer1::deep_crust_resonance::{
    resonance_social_spread_system, resonant_ore_exposure_system, ExcavationEvent,
    ResonantInfection, ResonantOre,
};
use scale::layer1::morale::Morale;
use scale::layer1::pop::Pop;
use scale::layer1::social::Relationships;

#[test]
fn test_deep_crust_resonance_integration() {
    let mut app = App::new();
    app.add_event::<ExcavationEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (
            resonant_ore_exposure_system,
            resonance_social_spread_system,
            deep_crust_resonance_chronicle_bridge,
        )
            .chain(),
    );

    let pop_a = app.world_mut().spawn((Pop, Morale::default())).id();

    let mut rels = Relationships::default();
    rels.affinities.insert(pop_a, 0.5); // pop_b likes pop_a
    let pop_b = app.world_mut().spawn((Pop, Morale::default(), rels)).id();

    let resonant_ore = app.world_mut().spawn(ResonantOre).id();

    app.world_mut()
        .resource_mut::<Events<ExcavationEvent>>()
        .send(ExcavationEvent {
            colony: Entity::PLACEHOLDER,
            miner: pop_a,
            discovery_type: "ResonantOre".to_string(),
            target: resonant_ore,
        });

    app.update();

    let morale_a = app.world().get::<Morale>(pop_a).unwrap();
    assert!(morale_a
        .modifiers
        .iter()
        .any(|m| m.label == "Deep Resonance"));
    assert!(app.world().get::<ResonantInfection>(pop_a).is_some());

    let morale_b = app.world().get::<Morale>(pop_b).unwrap();
    assert!(morale_b.modifiers.iter().any(|m| m.label == "Paranoia"));

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();
    assert_eq!(events.len(), 1);
    assert!(events[0].text.contains("Deep Crust Resonance"));
}
