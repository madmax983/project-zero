use bevy::prelude::*;
use scale::layer1::architecture::structure::Structure;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::energy::PowerSource;
use scale::layer1::nature::temperature::HeatSource;
use scale::layer2::weather::{AggroTarget, StormImpactEvent};
use scale::layer1::integration::{predatory_weather_emission_bridge_system, predatory_weather_impact_bridge_system};

#[test]
fn test_predatory_weather_emission_bridge() {
    let mut app = App::new();
    app.add_systems(Update, predatory_weather_emission_bridge_system);

    app.world_mut().spawn(PowerSource {
        output: 50.0,
        active: true,
    });

    app.world_mut().spawn(HeatSource {
        output: 30.0,
    });

    app.update();

    let mut query = app.world_mut().query::<&AggroTarget>();
    let target = query.iter(app.world()).next().expect("AggroTarget should be spawned");

    assert_eq!(target.energy_emission, 50.0);
    assert_eq!(target.heat_signature, 30.0);
}

#[test]
fn test_predatory_weather_impact_bridge() {
    let mut app = App::new();
    app.add_event::<StormImpactEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, predatory_weather_impact_bridge_system);

    let structure_entity = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(structure_entity).insert(Structure {
        current_hp: 100.0,
        max_hp: 100.0,
    });

    let dummy_storm = app.world_mut().spawn_empty().id();
    let dummy_target = app.world_mut().spawn_empty().id();

    app.world_mut().resource_mut::<Events<StormImpactEvent>>().send(StormImpactEvent {
        storm: dummy_storm,
        target: dummy_target,
        damage: 10.0,
    });

    app.update();

    let structure = app.world().get::<Structure>(structure_entity).unwrap();
    assert_eq!(structure.current_hp, 90.0, "Structure should take 10 damage");

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit chronicle event");
    assert_eq!(events[0].importance, EventImportance::Major);
}
