use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::health::Health;
use scale::layer1::pop::Pop;
use scale::layer1::resources::ColonyResources;
use scale::layer2::integration::{
    penal_funds_to_resources_system, prisoner_death_chronicle_bridge_system,
};
use scale::layer2::trade::penal_contracts::{
    check_prisoner_status_system, ColonyFunds, ContractTimer, FactionRelation, PenalContract,
    PrisonerDiedEvent, PrisonerOf,
};

#[test]
fn test_penal_funds_to_resources_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(
        Update,
        (
            scale::layer2::trade::penal_contracts::process_penal_contracts_system,
            penal_funds_to_resources_system,
        )
            .chain(),
    );

    app.insert_resource(ColonyResources::default());
    let _contract = app
        .world_mut()
        .spawn((
            PenalContract {
                payment_per_day: 10,
                prisoner_count: 5,
            },
            ContractTimer(1),
        ))
        .id();

    let _colony = app.world_mut().spawn(ColonyFunds(0)).id();

    app.update();

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(
        resources.credits, 50.0,
        "ColonyResources should receive 50 credits"
    );

    let mut query = app.world_mut().query::<&ColonyFunds>();
    let funds = query.single(app.world());
    assert_eq!(funds.0, 0, "ColonyFunds should be drained by the bridge");
}

#[test]
fn test_prisoner_death_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<PrisonerDiedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (
            check_prisoner_status_system,
            prisoner_death_chronicle_bridge_system,
        )
            .chain(),
    );

    let faction = app.world_mut().spawn(FactionRelation(100)).id();
    let _prisoner = app
        .world_mut()
        .spawn((
            Pop,
            PrisonerOf(faction),
            Health {
                current: 0.0,
                max: 100.0,
                has_rust_lung: false,
            },
        ))
        .id();

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(
        emitted[0].text.contains("State Prisoner"),
        "Event text should mention State Prisoner"
    );
}
