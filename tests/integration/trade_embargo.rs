use bevy::prelude::*;
use scale::layer1::resources::ResourceType;
use scale::layer1::social::factions::FactionId;
use scale::layer2::trade::routes::{
    execute_trade_routes_system, Colony, Timer, TradeRoute, TradeRouteExecutedEvent,
};
use scale::layer3::diplomacy::trade_embargoes::{apply_trade_embargo_system, TradeEmbargo};
use scale::layer3::market::GalacticMarket;

#[test]
fn test_trade_route_embargo_prevents_execution() {
    let mut app = App::new();
    app.add_systems(
        Update,
        (apply_trade_embargo_system, execute_trade_routes_system).chain(),
    );
    app.init_resource::<Events<TradeRouteExecutedEvent>>();

    let mut market = GalacticMarket::default();
    market.supply_pool.insert(ResourceType::Food, 100.0);
    app.insert_resource(market);

    app.world_mut().spawn(TradeEmbargo {
        resource: ResourceType::Food,
        enforcing_faction: FactionId::FarmersGuild,
    });

    let colony_a = app
        .world_mut()
        .spawn(Colony {
            name: "Earth".to_string(),
            resources: vec![("Food".to_string(), 500)],
        })
        .id();
    let colony_b = app
        .world_mut()
        .spawn(Colony {
            name: "Mars".to_string(),
            resources: vec![("Food".to_string(), 0)],
        })
        .id();

    let route = TradeRoute {
        source: colony_a,
        destination: colony_b,
        item_type: "Food".to_string(),
        amount: 100,
        interval: 1,
    };
    app.world_mut().spawn((route, Timer(1)));

    app.update(); // Set embargoes
    app.update(); // Execute trade routes

    // The route should NOT execute because Food is embargoed
    let events = app.world().resource::<Events<TradeRouteExecutedEvent>>();
    assert_eq!(
        events.get_cursor().len(events),
        0,
        "Trade route should be blocked by embargo"
    );

    let a_res = app
        .world()
        .get::<Colony>(colony_a)
        .unwrap()
        .get_resource("Food");
    assert_eq!(a_res, 500, "Source colony should still have its Food");
}
