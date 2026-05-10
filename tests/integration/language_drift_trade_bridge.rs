use bevy::prelude::*;
use scale::layer2::trade::routes::{Colony, Timer, TradeRoute, execute_trade_routes_system, TradeRouteExecutedEvent};
use scale::layer3::linguistic_drift::{LinguisticNetwork};
use scale::layer3::integration::language_drift_trade_bridge;

#[test]
fn test_translation_tax_reduces_trade_resources() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<TradeRouteExecutedEvent>();
    app.init_resource::<LinguisticNetwork>();

    app.add_systems(
        Update,
        (
            execute_trade_routes_system,
            language_drift_trade_bridge,
        )
            .chain(),
    );

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

    // Set linguistic drift to 500 (50% tax)
    app.world_mut()
        .resource_mut::<LinguisticNetwork>()
        .set_drift(colony_a, colony_b, 500.0);

    let route = TradeRoute {
        source: colony_a,
        destination: colony_b,
        item_type: "Food".to_string(),
        amount: 100,
        interval: 1,
    };
    app.world_mut().spawn((route, Timer(1)));

    app.update();

    // Colony A should lose 100 food (sent).
    // Colony B should receive 100, but then lose 50 due to 50% translation tax.
    let a_res = app.world().get::<Colony>(colony_a).unwrap().get_resource("Food");
    let b_res = app.world().get::<Colony>(colony_b).unwrap().get_resource("Food");

    assert_eq!(a_res, 400, "Source colony should have sent 100 Food");
    assert_eq!(b_res, 50, "Destination colony should only receive 50 Food due to 50% translation tax");
}
