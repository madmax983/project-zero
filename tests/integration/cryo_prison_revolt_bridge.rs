use bevy::prelude::*;
use scale::layer1::architecture::Structure;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::cryo_prison_sabotage_bridge_system;
use scale::layer1::cryo_prison::SabotageEvent;

#[test]
fn test_cryo_prison_sabotage_bridge() {
    let mut app = App::new();
    app.add_event::<SabotageEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, cryo_prison_sabotage_bridge_system);

    let structure_entity = app
        .world_mut()
        .spawn(Structure {
            current_hp: 100.0,
            max_hp: 100.0,
        })
        .id();

    let saboteur_entity = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(SabotageEvent {
        saboteur: saboteur_entity,
        facility: None,
    });

    app.update();

    // Verify structure took damage
    let structure = app.world().get::<Structure>(structure_entity).unwrap();
    assert!(structure.current_hp < 100.0);

    // Verify chronicle event was emitted
    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("sabotaged"));
}
