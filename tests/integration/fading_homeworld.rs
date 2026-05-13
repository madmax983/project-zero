use bevy::prelude::*;
use scale::layer1::economy::resources::ColonyResources;
use scale::layer3::diplomacy_reflection::{Civilization, DiplomaticRelations, DiplomaticStanding};
use scale::layer3::diplomacy::fading_homeworld::{
    generate_core_world_demand_system, handle_core_world_demands_system,
    update_core_world_decay_system, CoreWorld, CoreWorldDemandEvent, PlayerDemandResponse,
};
use scale::shared::time::SimulationTime;

fn setup_world() -> App {
    let mut app = App::new();
    app.init_resource::<SimulationTime>();
    app.init_resource::<Events<CoreWorldDemandEvent>>();
    app.init_resource::<Events<PlayerDemandResponse>>();
    app.insert_resource(ColonyResources {
        food: 500.0,
        ..Default::default()
    });

    app.add_systems(
        Update,
        (
            update_core_world_decay_system,
            generate_core_world_demand_system,
            handle_core_world_demands_system,
        )
            .chain(),
    );

    app
}

#[test]
fn test_core_world_generates_demands_on_decay() {
    let mut app = setup_world();

    let core_faction_id = app
        .world_mut()
        .spawn(CoreWorld {
            stability: 10.0, // Low stability
            decay_rate: 0.1,
        })
        .id();

    app.update();

    let events = app.world().resource::<Events<CoreWorldDemandEvent>>();
    let mut reader = events.get_reader();
    let emitted_events: Vec<_> = reader.read(events).collect();

    assert!(
        !emitted_events.is_empty(),
        "Low stability core world should generate a demand"
    );
    assert_eq!(emitted_events[0].demand.faction, core_faction_id);
    assert!(emitted_events[0].demand.amount > 0.0);
    assert!(emitted_events[0].demand.penalty > 0.0);
}

#[test]
fn test_player_can_fulfill_demands() {
    let mut app = setup_world();

    let core_faction_id = app
        .world_mut()
        .spawn((
            CoreWorld {
                stability: 100.0,
                decay_rate: 0.0,
            },
            Civilization {
                id: "Motherland".to_string(),
            },
            DiplomaticRelations {
                relations: vec![DiplomaticStanding {
                    target_id: "Colony".to_string(),
                    standing: 50.0,
                    sanctioned: false,
                }],
            },
        ))
        .id();

    // Trigger player response event directly
    app.world_mut().resource_mut::<Events<PlayerDemandResponse>>().send(PlayerDemandResponse {
        demand: scale::layer3::diplomacy::fading_homeworld::CoreWorldDemand {
            amount: 200.0,
            penalty: 20.0,
            faction: core_faction_id,
        },
        accept: true,
    });

    app.update();

    // Verify resources were depleted
    let pool = app.world().resource::<ColonyResources>();
    assert!((pool.food - 300.0).abs() < f32::EPSILON);

    // Verify standing was maintained
    let relation = app.world().get::<DiplomaticRelations>(core_faction_id).unwrap();
    assert!((relation.relations[0].standing - 50.0).abs() < f32::EPSILON);
}

#[test]
fn test_player_can_refuse_demands() {
    let mut app = setup_world();

    let core_faction_id = app
        .world_mut()
        .spawn((
            CoreWorld {
                stability: 100.0,
                decay_rate: 0.0,
            },
            Civilization {
                id: "Motherland".to_string(),
            },
            DiplomaticRelations {
                relations: vec![DiplomaticStanding {
                    target_id: "Colony".to_string(),
                    standing: 50.0,
                    sanctioned: false,
                }],
            },
        ))
        .id();

    // Trigger player response event directly
    app.world_mut().resource_mut::<Events<PlayerDemandResponse>>().send(PlayerDemandResponse {
        demand: scale::layer3::diplomacy::fading_homeworld::CoreWorldDemand {
            amount: 200.0,
            penalty: 20.0,
            faction: core_faction_id,
        },
        accept: false,
    });

    app.update();

    // Verify resources were maintained
    let pool = app.world().resource::<ColonyResources>();
    assert!((pool.food - 500.0).abs() < f32::EPSILON);

    // Verify standing decreased
    let relation = app.world().get::<DiplomaticRelations>(core_faction_id).unwrap();
    assert!((relation.relations[0].standing - 30.0).abs() < f32::EPSILON);
}
