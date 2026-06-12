use bevy::prelude::*;
use scale::layer1::resources::ResourceType;
use scale::layer2::trade::routes::{execute_trade_routes_system, Colony, Timer, TradeRoute, TradeRouteExecutedEvent};
use scale::layer3::integration::trade_route_market_bridge_system;
use scale::layer3::market::GalacticMarket;

#[test]
fn test_trade_route_market_bridge() {
    let mut app = App::new();
    app.add_event::<TradeRouteExecutedEvent>();
    app.add_systems(
        Update,
        (execute_trade_routes_system, trade_route_market_bridge_system).chain(),
    );

    let mut market = GalacticMarket::default();
    market.supply_pool.insert(ResourceType::Food, 100.0);
    app.insert_resource(market);

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

    app.update();

    let market = app.world().resource::<GalacticMarket>();
    let pool = market.supply_pool.get(&ResourceType::Food).copied().unwrap_or(0.0);
    assert_eq!(
        pool, 200.0,
        "Market supply pool should increase by the traded amount (100 + 100 = 200)"
    );
}
