
use scale::layer2::integration::orbital_secession_embargo_bridge_system;
use scale::layer2::orbit::secession::SecessionState;
use scale::layer2::fleet::FleetFaction;
use scale::layer3::diplomacy::trade_embargoes::TradeEmbargo;
use scale::layer1::resources::ResourceType;
use scale::layer1::social::factions::FactionId;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use bevy::prelude::*;

#[test]
fn test_orbital_secession_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, orbital_secession_embargo_bridge_system);

    let habitat = app.world_mut().spawn((
        SecessionState::Loyal,
        FleetFaction::Player,
    )).id();

    app.update();

    // Change state to Seceded
    app.world_mut().entity_mut(habitat).insert(SecessionState::Seceded);

    app.update();

    // Verify faction changed to Pirate
    let faction = app.world().get::<FleetFaction>(habitat).unwrap();
    assert_eq!(*faction, FleetFaction::Pirate);

    // Verify TradeEmbargoes were spawned
    let mut embargo_query = app.world_mut().query::<&TradeEmbargo>();
    let mut food_embargo = false;
    let mut metal_embargo = false;

    for embargo in embargo_query.iter(app.world()) {
        if embargo.resource == ResourceType::Food && embargo.enforcing_faction == FactionId::Stateless {
            food_embargo = true;
        }
        if embargo.resource == ResourceType::Metal && embargo.enforcing_faction == FactionId::Stateless {
            metal_embargo = true;
        }
    }

    assert!(food_embargo, "Food embargo should be spawned");
    assert!(metal_embargo, "Metal embargo should be spawned");

    // Verify Chronicle event
    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut event_found = false;

    for event in reader.read(events) {
        if event.importance == EventImportance::Major && event.text.contains("embargoing the planet") {
            event_found = true;
            break;
        }
    }

    assert!(event_found, "Major chronicle event should be fired");
}
