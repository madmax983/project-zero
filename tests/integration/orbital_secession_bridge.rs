use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::economy::ColonyResources;
use scale::layer1::pop::Pop;
use scale::layer1::resources::ResourceType;
use scale::layer1::social::factions::FactionId;
use scale::layer2::integration::orbital_secession_chronicle_bridge;
use scale::layer2::orbit::secession::{
    evaluate_orbital_secession_system, OrbitalHabitat, SecessionState, Unrest,
};
use scale::layer3::diplomacy::trade_embargoes::TradeEmbargo;

#[test]
fn test_orbital_secession_bridge_triggers_events() {
    let mut app = App::new();

    app.add_systems(
        Update,
        (
            evaluate_orbital_secession_system,
            orbital_secession_chronicle_bridge,
        )
            .chain(),
    );

    app.add_event::<AddChronicleEvent>();

    // Spawn an orbital habitat that meets secession criteria
    let habitat = app
        .world_mut()
        .spawn((
            OrbitalHabitat {
                population: 5000,
                wealth: 100000.0,
            },
            Unrest { level: 90.0 },
        ))
        .id();

    // Spawn a colony with fewer resources
    for _ in 0..1000 {
        app.world_mut().spawn(Pop);
    }
    app.world_mut().insert_resource(ColonyResources {
        credits: 50000.0,
        ..Default::default()
    });

    app.update();

    // Verify secession state
    let state = app.world().get::<SecessionState>(habitat).unwrap();
    assert_eq!(*state, SecessionState::Seceded);

    // Verify Chronicle Event
    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.get_cursor().len(events), 1);

    // Verify Embargoes
    let mut embargoes = app.world_mut().query::<&TradeEmbargo>();
    let mut found_food = false;
    let mut found_metal = false;

    for embargo in embargoes.iter(app.world()) {
        if embargo.enforcing_faction == FactionId::OrbitalSecessionist {
            if embargo.resource == ResourceType::Food {
                found_food = true;
            } else if embargo.resource == ResourceType::Metal {
                found_metal = true;
            }
        }
    }

    assert!(found_food, "Food embargo should be declared");
    assert!(found_metal, "Metal embargo should be declared");
}
