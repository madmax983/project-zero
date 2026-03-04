use bevy_ecs::prelude::*;
use scale::layer1::map::GridPosition;
use scale::layer1::resources::{ColonyResources, ResourceType};
use scale::layer1::shadow_market::ShadowTrader;
use scale::layer1::trade::{execute_trade, TradeDeal};

#[test]
fn shadow_market_trade_success() {
    let mut world = World::new();
    let resources = ColonyResources {
        food: 50.0,
        ..Default::default()
    };
    world.insert_resource(resources);

    let deal = TradeDeal {
        cost_resource: ResourceType::Food,
        cost_amount: 10.0,
        give_resource: ResourceType::Alcohol,
        give_amount: 5.0,
    };

    world.spawn((
        ShadowTrader {
            merchant: scale::layer1::trade::Merchant {
                name: "Shadow Trader".to_string(),
                arrival_tick: 0,
                departure_tick: 500,
                deals: vec![deal.clone()],
            },
        },
        GridPosition { x: 5, y: 5 },
    ));

    // Player selects the shadow trader entity and executes the trade.
    let success = execute_trade(&mut world, &deal);

    assert!(success);

    let res = world.resource::<ColonyResources>();
    assert!((res.food - 40.0).abs() < f32::EPSILON);
    assert!((res.alcohol - 5.0).abs() < f32::EPSILON);
}
