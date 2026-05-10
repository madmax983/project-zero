use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::temporal_chamber_power_bridge_system;
use scale::layer1::resources::ColonyResources;
use scale::layer1::temporal_chamber::TemporalChamber;

#[test]
fn test_temporal_chamber_power_bridge_consumes_fuel() {
    let mut app = App::new();
    app.add_systems(Update, temporal_chamber_power_bridge_system);
    app.init_resource::<Events<AddChronicleEvent>>();

    let resources = ColonyResources {
        fuel: 100.0,
        ..Default::default()
    };
    app.insert_resource(resources);

    app.world_mut().spawn(TemporalChamber {
        time_dilation_factor: 0.1,
        active: true,
        energy_cost: 20.0,
        ticks_active: 0,
    });

    app.update();

    let res = app.world().resource::<ColonyResources>();
    assert_eq!(
        res.fuel, 80.0,
        "Should consume fuel equal to the energy cost."
    );

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert!(
        events.is_empty(),
        "Should not emit shockwave if fuel is sufficient."
    );
}

#[test]
fn test_temporal_chamber_power_bridge_shockwave_on_failure() {
    let mut app = App::new();
    app.add_systems(Update, temporal_chamber_power_bridge_system);
    app.init_resource::<Events<AddChronicleEvent>>();

    let resources = ColonyResources {
        fuel: 10.0,
        ..Default::default()
    }; // insufficient
    app.insert_resource(resources);

    let chamber_entity = app
        .world_mut()
        .spawn(TemporalChamber {
            time_dilation_factor: 0.1,
            active: true,
            energy_cost: 20.0,
            ticks_active: 0,
        })
        .id();

    app.update();

    let res = app.world().resource::<ColonyResources>();
    assert_eq!(
        res.fuel, 10.0,
        "Should not consume partial fuel on failure."
    );

    let chamber = app.world().get::<TemporalChamber>(chamber_entity).unwrap();
    assert!(!chamber.active, "Chamber should be deactivated.");

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();
    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(
        emitted[0].text,
        "Temporal shockwave released due to power failure in echo chamber!"
    );
}
