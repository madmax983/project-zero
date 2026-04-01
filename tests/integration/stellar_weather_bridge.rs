use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer2::fleet::{FleetComposition, FleetHealth};
use scale::layer2::integration::stellar_weather_damage_bridge_system;
use scale::layer2::navigation::stellar_weather::FleetDamagedEvent;
use scale::layer2::ship::{Ship, ShipType};
use scale::shared::time::SimulationTime;

#[test]
fn test_stellar_weather_damage_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SimulationTime>();
    app.add_event::<FleetDamagedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, stellar_weather_damage_bridge_system);

    let fleet_entity = app
        .world_mut()
        .spawn((
            FleetHealth {
                current: 100.0,
                max: 100.0,
            },
            FleetComposition {
                ships: vec![Ship {
                    ship_type: ShipType::Scout,
                    health: 100.0,
                    max_health: 100.0,
                }],
            },
        ))
        .id();

    // Send a damage event that reduces health but doesn't destroy
    app.world_mut().send_event(FleetDamagedEvent {
        fleet: fleet_entity,
        amount: 50.0,
    });

    app.update();

    let health = app.world().get::<FleetHealth>(fleet_entity).unwrap();
    assert_eq!(health.current, 50.0, "Fleet health should be reduced");

    let comp = app.world().get::<FleetComposition>(fleet_entity).unwrap();
    assert_eq!(comp.ships[0].health, 50.0, "Ship health should be reduced");

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();
    assert_eq!(
        events.len(),
        1,
        "Should emit exactly one AddChronicleEvent for the damage"
    );

    // Send a damage event that destroys the fleet
    app.world_mut().send_event(FleetDamagedEvent {
        fleet: fleet_entity,
        amount: 100.0,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    // Total 3 events because the first update added 1, second update adds 2 (damage + destruction)
    assert_eq!(
        events.len(),
        3,
        "Should emit damage and destruction events"
    );

    assert!(
        app.world().get_entity(fleet_entity).is_err(),
        "Fleet should be despawned when health hits zero"
    );
}
