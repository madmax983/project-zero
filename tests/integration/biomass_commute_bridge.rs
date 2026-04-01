use bevy::prelude::*;
use scale::layer1::logistics::biomass_network::{
    digest_transit_contents, process_biomass_network_hunger, BiomassNetwork, InTransit,
};
use scale::layer1::pop::{Pop, PopDied, PopName};

#[test]
fn test_biomass_commute_digestion_emits_pop_died() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_event::<PopDied>();

    app.add_systems(
        Update,
        (process_biomass_network_hunger, digest_transit_contents).chain(),
    );

    let network_entity = app
        .world_mut()
        .spawn(BiomassNetwork {
            hunger: 100.0,
            max_hunger: 100.0,
            consumption_rate: 10.0,
        })
        .id();

    let pop_entity = app
        .world_mut()
        .spawn((
            InTransit {
                network: network_entity,
            },
            Pop,
            PopName("Unlucky Commuter".to_string()),
        ))
        .id();

    app.update();

    let events = app.world().resource::<Events<PopDied>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<&PopDied> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one PopDied event");
    assert_eq!(emitted[0].entity, pop_entity, "The digested pop entity should match the event");
}
