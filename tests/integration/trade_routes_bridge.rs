use bevy::prelude::*;
use scale::layer1::resources::ColonyResources;
use scale::layer2::integration::{
    post_trade_route_sync_system, pre_trade_route_sync_system, HomeColony,
};
use scale::layer2::trade::routes::{execute_trade_routes_system, Colony, Timer, TradeRoute};

#[test]
fn test_trade_route_resources_bridge_incoming() {
    let mut app = App::new();
        app.add_event::<scale::layer2::trade::routes::TradeRouteExecutedEvent>();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_systems(
        Update,
        (
            pre_trade_route_sync_system,
            execute_trade_routes_system,
            post_trade_route_sync_system,
        )
            .chain(),
    );

    app.insert_resource(ColonyResources::default());

    let colony_a = app
        .world_mut()
        .spawn(Colony {
            name: "Earth".to_string(),
            resources: vec![("Food".to_string(), 500)],
        })
        .id();

    // Home colony
    let colony_b = app
        .world_mut()
        .spawn((
            Colony {
                name: "Mars".to_string(),
                resources: vec![],
            },
            HomeColony,
        ))
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

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(
        resources.food, 110.0,
        "Food should have been added to ColonyResources"
    );
}

#[test]
fn test_trade_route_resources_bridge_outgoing() {
    let mut app = App::new();
        app.add_event::<scale::layer2::trade::routes::TradeRouteExecutedEvent>();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_systems(
        Update,
        (
            pre_trade_route_sync_system,
            execute_trade_routes_system,
            post_trade_route_sync_system,
        )
            .chain(),
    );

    let resources = ColonyResources {
        metal: 500.0,
        ..Default::default()
    };
    app.insert_resource(resources);

    let colony_a = app
        .world_mut()
        .spawn((
            Colony {
                name: "Mars".to_string(),
                resources: vec![],
            },
            HomeColony,
        ))
        .id();

    let colony_b = app
        .world_mut()
        .spawn(Colony {
            name: "Earth".to_string(),
            resources: vec![],
        })
        .id();

    let route = TradeRoute {
        source: colony_a,
        destination: colony_b,
        item_type: "Metal".to_string(),
        amount: 100,
        interval: 1,
    };
    app.world_mut().spawn((route, Timer(1)));

    app.update();

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(
        resources.metal, 400.0,
        "Metal should be deducted from ColonyResources"
    );

    let earth_colony = app.world().get::<Colony>(colony_b).unwrap();
    assert_eq!(
        earth_colony.get_resource("Metal"),
        100,
        "Earth should have received the Metal"
    );
}
