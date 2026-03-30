with open("src/layer1/integration.rs", "r") as f:
    content = f.read()

content = content.replace(
    "    hubs: Query<\n        (&GridPosition, &crate::layer1::energy::PowerConsumer),\n        With<crate::layer1::drone::DroneHub>,\n    >,",
    "    hubs: Query<\n        (Entity, &GridPosition, &crate::layer1::energy::PowerConsumer),\n        With<crate::layer1::drone::DroneHub>,\n    >,"
)

content = content.replace(
    "    for (pos, power) in hubs.iter() {\n        if power.active {\n            // Spawn drone\n            commands.spawn((\n                crate::layer1::drone::Drone,\n                *pos,\n                crate::layer1::utility_ai::PopAction::default(),\n                crate::layer1::drone::DroneBattery {\n                    current: 100.0,\n                    max: 100.0,\n                },\n                crate::layer1::pop::Speed {\n                    base: 1.0,\n                    current: 1.0,\n                    accumulator: 0.0,\n                },\n                crate::layer1::utility_ai::UtilityWeights::default(),\n            ));\n            break; // Only one per tick\n        }\n    }",
    "    for (entity, pos, power) in hubs.iter() {\n        if power.active {\n            // Spawn drone\n            commands.spawn((\n                crate::layer1::drone::Drone,\n                crate::layer1::drone::ParentHub(entity),\n                *pos,\n                crate::layer1::utility_ai::PopAction::default(),\n                crate::layer1::drone::DroneBattery {\n                    current: 100.0,\n                    max: 100.0,\n                },\n                crate::layer1::pop::Speed {\n                    base: 1.0,\n                    current: 1.0,\n                    accumulator: 0.0,\n                },\n                crate::layer1::utility_ai::UtilityWeights::default(),\n            ));\n            break; // Only one per tick\n        }\n    }"
)

with open("src/layer1/integration.rs", "w") as f:
    f.write(content)
