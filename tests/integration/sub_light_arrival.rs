use bevy::prelude::*;
use bevy::app::App;
use scale::layer2::sub_light_arrival::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance, };
use scale::shared::time::SimulationTime;
use scale::layer2::integration::sub_light_arrival_chronicle_bridge_system;

#[test]
fn test_sub_light_arrival_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<SubLightArrivalEvent>();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(SimulationTime::default());

    app.add_systems(Update, sub_light_arrival_chronicle_bridge_system);

    app.world_mut().send_event(SubLightArrivalEvent {
        ship_entity: Entity::PLACEHOLDER,
        faction_id: FactionId::from_str("ancient_empire"),
        arrival_system_id: SystemId::from_str("capital"),
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit a chronicle event");
    assert!(emitted[0].text.contains("ancient_empire"), "Should mention the faction");
    assert!(emitted[0].text.contains("capital"), "Should mention the system");
    assert_eq!(emitted[0].importance, EventImportance::Major, "Should be Major event");
}
