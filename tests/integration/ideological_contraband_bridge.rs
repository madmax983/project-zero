use bevy::prelude::*;
use scale::layer1::economy::ideological_contraband::{apply_cultural_contraband_system, Ethics};
use scale::layer1::economy::TradeImportEvent;
use scale::layer1::pop::Pop;
use scale::layer2::integration::{ideological_contraband_route_bridge, HomeColony};
use scale::layer2::trade::routes::{execute_trade_routes_system, Colony, Timer, TradeRoute};

#[test]
fn test_trade_route_ideological_contraband_bridge() {
    let mut app = App::new();
    app.add_event::<scale::layer2::trade::routes::TradeRouteExecutedEvent>();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_systems(
        Update,
        (
            execute_trade_routes_system,
            ideological_contraband_route_bridge.after(execute_trade_routes_system),
            apply_cultural_contraband_system.after(ideological_contraband_route_bridge),
        ),
    );
    app.add_event::<TradeImportEvent>();

    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            Ethics {
                collectivism: 0,
                elitism: 0,
            },
        ))
        .id();

    let colony_a = app
        .world_mut()
        .spawn(Colony {
            name: "Earth".to_string(),
            resources: vec![("Worker Boots".to_string(), 500)],
        })
        .id();

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

    app.world_mut().spawn((
        TradeRoute {
            source: colony_a,
            destination: colony_b,
            item_type: "Worker Boots".to_string(),
            amount: 100,
            interval: 1,
        },
        Timer(1),
    ));

    app.update();

    let ethics = app.world().get::<Ethics>(pop_entity).unwrap();
    assert!(
        ethics.collectivism > 0,
        "Collectivism should increase due to the imported Worker Boots"
    );
}
