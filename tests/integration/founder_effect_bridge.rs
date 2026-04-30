use bevy::prelude::*;
use scale::layer1::pop::{Pop, PopBorn};
use scale::layer1::psychology::traits::{Trait, Traits};
use scale::layer2::culture::founder_effect::ColonyCulture;
use scale::layer2::integration::founder_effect_bridge_system;
use std::collections::HashMap;

#[test]
fn test_founder_effect_bridge_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, founder_effect_bridge_system);
    app.add_event::<PopBorn>();

    // Spawn the ColonyCulture with a dominant trait
    app.world_mut().spawn(ColonyCulture {
        dominant_trait: Trait::StoneSkin,
        trait_distribution: HashMap::new(),
    });

    // Spawn a pop with no traits initially
    let pop_entity = app
        .world_mut()
        .spawn((Pop, Traits(bevy::utils::HashSet::new())))
        .id();

    // Fire the PopBorn event
    app.world_mut()
        .resource_mut::<Events<PopBorn>>()
        .send(PopBorn {
            entity: pop_entity,
            name: "Test Pop".to_string(),
            tick: 0,
            source: "Test".to_string(),
        });

    app.update();

    // Verify the pop acquired the StoneSkin trait
    let pop_traits = app.world().get::<Traits>(pop_entity).unwrap();
    assert!(
        pop_traits.has(Trait::StoneSkin),
        "Pop should have acquired the dominant trait"
    );
}
