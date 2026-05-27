use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer2::fleet::{Fleet, InOrbit};
use scale::layer2::integration::orbital_drydock_fleet_bridge_system;
use scale::layer2::station::{ShipConstructionCompletedEvent, Station, StationType};

#[test]
fn test_orbital_drydock_fleet_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<ShipConstructionCompletedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, orbital_drydock_fleet_bridge_system);

    let drydock_entity = app
        .world_mut()
        .spawn(Station {
            station_type: StationType::OrbitalDrydock,
        })
        .id();

    app.world_mut()
        .resource_mut::<Events<ShipConstructionCompletedEvent>>()
        .send(ShipConstructionCompletedEvent {
            drydock_entity,
            ship_class: "Dreadnought".to_string(),
        });

    app.update();

    let mut fleet_query = app.world_mut().query::<(&Fleet, &InOrbit)>();
    let mut fleet_spawned = false;
    for (_fleet, in_orbit) in fleet_query.iter(app.world()) {
        if in_orbit.parent == drydock_entity {
            fleet_spawned = true;
        }
    }
    assert!(
        fleet_spawned,
        "Fleet should be spawned with InOrbit attached."
    );

    let chron_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chron_events.get_cursor();
    let mut chron_emitted = false;
    for ev in reader.read(chron_events) {
        if ev.text.contains("Dreadnought") {
            chron_emitted = true;
        }
    }
    assert!(chron_emitted, "AddChronicleEvent should be emitted.");
}
