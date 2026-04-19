use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::economy::remittances::MigrantArrivalEvent;
use scale::layer1::integration::{
    beacon_migrant_arrival_bridge, beacon_pirate_raid_bridge, beacon_trade_ship_bridge,
};
use scale::layer1::morale::Morale;
use scale::layer1::pop::Pop;
use scale::layer1::resources::ColonyResources;
use scale::layer1::trade::MerchantState;
use scale::layer1::void_weed::PirateRaidEvent;
use scale::layer2::trade::blockade::TradeShipArrivalEvent;
use scale::shared::time::SimulationTime;

#[test]
fn test_beacon_migrant_arrival_bridge() {
    let mut app = App::new();
    app.insert_resource(SimulationTime {
        tick: 100,
        ..Default::default()
    });
    app.add_event::<MigrantArrivalEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, beacon_migrant_arrival_bridge);

    app.world_mut().send_event(MigrantArrivalEvent {
        home_faction: Entity::PLACEHOLDER,
        count: 3,
        criminal_chance: 0.0,
        low_skill_chance: 0.0,
    });

    app.update();

    let mut query = app.world_mut().query::<&Pop>();
    let pops = query.iter(app.world()).count();
    assert_eq!(pops, 3, "Bridge should spawn the correct number of pops");
}

#[test]
fn test_beacon_trade_ship_bridge() {
    let mut app = App::new();
    app.insert_resource(SimulationTime {
        tick: 100,
        ..Default::default()
    });
    app.insert_resource(MerchantState::default());
    app.add_event::<TradeShipArrivalEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, beacon_trade_ship_bridge);

    app.world_mut().send_event(TradeShipArrivalEvent {
        cargo_value: 5000.0,
        faction: "Test Faction".to_string(),
    });

    app.update();

    let state = app.world().resource::<MerchantState>();
    assert!(
        state.active_merchant.is_some(),
        "Bridge should force a merchant to arrive"
    );
    assert_eq!(
        state.active_merchant.as_ref().unwrap().name,
        "Test Faction Ship"
    );
}

#[test]
fn test_beacon_pirate_raid_bridge() {
    let mut app = App::new();
    app.insert_resource(SimulationTime {
        tick: 100,
        ..Default::default()
    });
    app.insert_resource(ColonyResources {
        food: 100.0,
        metal: 50.0,
        ..Default::default()
    });
    app.add_event::<PirateRaidEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, beacon_pirate_raid_bridge);

    // Spawn a pop to test morale drop
    let pop = app.world_mut().spawn((Pop, Morale::default())).id();

    app.world_mut().send_event(PirateRaidEvent);

    app.update();

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.food, 50.0, "Pirates should steal 50 food");
    assert_eq!(resources.metal, 30.0, "Pirates should steal 20 metal");

    let morale = app.world().get::<Morale>(pop).unwrap();
    assert!(
        morale.modifiers.iter().any(|m| m.label == "Pirate Raid"),
        "Morale modifier should be added"
    );
}
