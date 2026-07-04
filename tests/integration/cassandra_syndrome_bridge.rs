use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::cassandra_syndrome::cassandra_syndrome_chronicle_bridge;
use scale::layer1::cassandra_syndrome::{CultLeader, DoomsdayWarningEvent, Prophetic};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::environment::disasters::DisasterType;
use scale::layer1::pop::Pop;

#[test]
fn test_cassandra_syndrome_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<DoomsdayWarningEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, cassandra_syndrome_chronicle_bridge);

    let pop = app
        .world_mut()
        .spawn((Pop, Prophetic { cooldown: 0.0 }))
        .id();

    app.world_mut()
        .resource_mut::<Events<DoomsdayWarningEvent>>()
        .send(DoomsdayWarningEvent {
            prophet_entity: pop,
            disaster_type: DisasterType::MassiveEarthquake,
        });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("Doomsday Warning"));
}

#[test]
fn test_cassandra_syndrome_cult_leader_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(
        Update,
        scale::layer1::cassandra_syndrome::cassandra_cult_chronicle_bridge,
    );

    app.world_mut().spawn((Pop, CultLeader));

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("Cult Leader"));
}
