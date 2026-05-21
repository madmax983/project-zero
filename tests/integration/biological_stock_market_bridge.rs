use bevy::prelude::*;
use scale::layer1::resources::ResourceType;
use scale::layer2::trade::routes::TradeRouteExecutedEvent;
use scale::layer3::economy::biological_stock_market::biological_stock_market_bridge;
use scale::layer3::market::GalacticMarket;

#[test]
fn test_infected_imports_increase_demand() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<TradeRouteExecutedEvent>();
    app.init_resource::<GalacticMarket>();

    app.add_systems(Update, biological_stock_market_bridge);

    app.world_mut()
        .resource_mut::<GalacticMarket>()
        .supply_pool
        .insert(ResourceType::Food, 1000.0);

    app.world_mut().send_event(TradeRouteExecutedEvent {
        source: Entity::from_raw(1),
        destination: Entity::from_raw(2),
        item_type: "Food".to_string(),
        amount: 100,
    });

    app.update();

    let market = app.world().resource::<GalacticMarket>();
    let pool = market.supply_pool.get(&ResourceType::Food).unwrap();
    assert_eq!(
        *pool, 995.0,
        "Supply pool should decrease by 5% of imported amount, simulating demand increase"
    );
}
