import os

path = "tests/integration/trade_routes_bridge.rs"
with open(path, "r") as f:
    content = f.read()

merge_diff = """<<<<<<< SEARCH
    app.add_event::<scale::layer2::trade::routes::TradeRouteExecutedEvent>();
    app.insert_resource(ColonyResources::default());
=======
    app.world_mut().init_resource::<bevy::ecs::event::Events<scale::layer2::trade::routes::TradeRouteExecutedEvent>>();
    app.insert_resource(ColonyResources::default());
>>>>>>> REPLACE"""

with open("diff.txt", "w") as f:
    f.write(merge_diff)

path = "tests/integration/trade_routes_bridge.rs"
with open(path, "r") as f:
    content = f.read()

merge_diff = """<<<<<<< SEARCH
    app.add_event::<scale::layer2::trade::routes::TradeRouteExecutedEvent>();
    let resources = ColonyResources {
        metal: 500.0,
        ..Default::default()
    };
    app.insert_resource(resources);
=======
    app.world_mut().init_resource::<bevy::ecs::event::Events<scale::layer2::trade::routes::TradeRouteExecutedEvent>>();
    let resources = ColonyResources {
        metal: 500.0,
        ..Default::default()
    };
    app.insert_resource(resources);
>>>>>>> REPLACE"""
with open("diff2.txt", "w") as f:
    f.write(merge_diff)

path3 = "tests/integration/ideological_contraband_bridge.rs"
with open(path3, "r") as f:
    content3 = f.read()

merge_diff3 = """<<<<<<< SEARCH
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_event::<scale::layer2::trade::routes::TradeRouteExecutedEvent>();
=======
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.world_mut().init_resource::<bevy::ecs::event::Events<scale::layer2::trade::routes::TradeRouteExecutedEvent>>();
>>>>>>> REPLACE"""
with open("diff3.txt", "w") as f:
    f.write(merge_diff3)
