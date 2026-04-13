import re

with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

content = content.replace("query: bevy_ecs::prelude::Query<(bevy_ecs::prelude::Entity,     query: bevy_ecs::prelude::Query<(bevy_ecs::prelude::Entity, &crate::layer1::flora::PheromoneFlora), bevy_ecs::prelude::Or<(bevy_ecs::prelude::Added<crate::layer1::flora::PheromoneFlora>, bevy_ecs::prelude::Changed<crate::layer1::flora::PheromoneFlora>)>>,crate::layer1::flora::PheromoneFlora), bevy_ecs::prelude::Or<(bevy_ecs::prelude::Added<crate::layer1::flora::PheromoneFlora>, bevy_ecs::prelude::Changed<crate::layer1::flora::PheromoneFlora>)>>", "query: bevy_ecs::prelude::Query<(bevy_ecs::prelude::Entity, &crate::layer1::flora::PheromoneFlora), bevy_ecs::prelude::Or<(bevy_ecs::prelude::Added<crate::layer1::flora::PheromoneFlora>, bevy_ecs::prelude::Changed<crate::layer1::flora::PheromoneFlora>)>>")

with open("src/layer1/core/integration.rs", "w") as f:
    f.write(content)
