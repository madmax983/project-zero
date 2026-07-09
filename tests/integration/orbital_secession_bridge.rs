use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::resources::ResourceType;
use scale::layer1::social::factions::FactionId as L1FactionId;
use scale::layer2::orbit::secession::SecessionState;
use scale::layer3::diplomacy::trade_embargoes::TradeEmbargo;
use scale::layer2::integration::orbital_secession_embargo_bridge;

#[test]
fn test_orbital_secession_embargo_bridge() {
    let mut app = App::new();
    app.add_systems(Update, orbital_secession_embargo_bridge);
    app.init_resource::<Events<AddChronicleEvent>>();

    // Create an entity that just seceded
    app.world_mut().spawn(SecessionState::Seceded);

    app.update();

    // Verify TradeEmbargo component is created
    let mut embargo_query = app.world_mut().query::<&TradeEmbargo>();
    let embargo_count = embargo_query.iter(app.world()).count();
    assert_eq!(embargo_count, 1, "A TradeEmbargo should be spawned upon secession");

    let embargo = embargo_query.single(app.world());
    assert_eq!(embargo.resource, ResourceType::Metal);
    assert_eq!(embargo.enforcing_faction, L1FactionId::Unaligned); // Using Unaligned as a mock new faction

    // Verify AddChronicleEvent is emitted
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "One AddChronicleEvent should be emitted");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("A massive orbital habitat has seceded"));
}
