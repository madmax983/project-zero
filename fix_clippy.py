with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

# Need to place allow before the function or define a type
content = content.replace("#[allow(clippy::type_complexity)]\n    query", "query")

content = content.replace(
    "pub fn flora_scent_bridge_system(\n    mut commands: bevy_ecs::prelude::Commands,\n    query: bevy_ecs::prelude::Query<(bevy_ecs::prelude::Entity, &crate::layer1::flora::PheromoneFlora), bevy_ecs::prelude::Or<(bevy_ecs::prelude::Added<crate::layer1::flora::PheromoneFlora>, bevy_ecs::prelude::Changed<crate::layer1::flora::PheromoneFlora>)>>,\n)",
    "#[allow(clippy::type_complexity)]\npub fn flora_scent_bridge_system(\n    mut commands: bevy_ecs::prelude::Commands,\n    query: bevy_ecs::prelude::Query<(bevy_ecs::prelude::Entity, &crate::layer1::flora::PheromoneFlora), bevy_ecs::prelude::Or<(bevy_ecs::prelude::Added<crate::layer1::flora::PheromoneFlora>, bevy_ecs::prelude::Changed<crate::layer1::flora::PheromoneFlora>)>>,\n)"
)

with open("src/layer1/core/integration.rs", "w") as f:
    f.write(content)
