use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::social::factions::{FactionMember, FactionId};
use scale::layer2::cryo_mutiny::{CryoShipEvent, MutinyTracker, trigger_cryo_ship_landing_system, MutineerPop};
use scale::layer2::integration::cryo_mutiny_bridge_system;

#[test]
fn test_cryo_mutiny_integration() {
    let mut app = App::new();

    app.add_event::<AddChronicleEvent>();
    app.init_resource::<MutinyTracker>();
    app.init_resource::<CryoShipEvent>();

    app.add_systems(Update, (trigger_cryo_ship_landing_system, cryo_mutiny_bridge_system).chain());

    // Trigger mutiny
    app.world_mut().resource_mut::<CryoShipEvent>().active = true;
    app.update();

    // Verify pops were assigned to the Mutineer faction
    let mut mutineers_query = app.world_mut().query_filtered::<&FactionMember, With<MutineerPop>>();
    let mut count = 0;
    for faction in mutineers_query.iter(app.world()) {
        assert_eq!(faction.faction_id, Some(FactionId::CryoMutineers)); // Using Cartel as generic hostile or add new?
        count += 1;
    }
    assert_eq!(count, 10, "Should have 10 mutineers with faction assigned");

    // Verify chronicle event
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected one chronicle event");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("Cryo-Mutiny"));
}
