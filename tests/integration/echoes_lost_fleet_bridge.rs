use bevy::prelude::*;
use scale::layer1::anomalies::echoes_lost_fleet::LuredByGhostFleet;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::lured_pops_escape_bridge_system;

#[test]
fn test_lured_pops_escape_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Events<AddChronicleEvent>>();
    app.add_systems(Update, lured_pops_escape_bridge_system);

    let pop = app.world_mut().spawn(LuredByGhostFleet).id();

    app.update();

    assert!(app.world().get_entity(pop).is_err());

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("Ghost Dreadnought"));
}
