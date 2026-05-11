import os

path = "tests/integration/trade_routes_bridge.rs"
with open(path, "r") as f:
    content = f.read()

content = content.replace("app.insert_resource(ColonyResources::default());", "app.init_resource::<bevy_ecs::event::Events<scale::layer2::trade::routes::TradeRouteExecutedEvent>>();\n    app.insert_resource(ColonyResources::default());")
content = content.replace("app.insert_resource(resources);", "app.init_resource::<bevy_ecs::event::Events<scale::layer2::trade::routes::TradeRouteExecutedEvent>>();\n    app.insert_resource(resources);")

with open(path, "w") as f:
    f.write(content)

path2 = "tests/integration/ideological_contraband_bridge.rs"
with open(path2, "r") as f:
    content2 = f.read()

content2 = content2.replace("app.add_plugins(bevy::MinimalPlugins);", "app.add_plugins(bevy::MinimalPlugins);\n    app.init_resource::<bevy_ecs::event::Events<scale::layer2::trade::routes::TradeRouteExecutedEvent>>();")

with open(path2, "w") as f:
    f.write(content2)
