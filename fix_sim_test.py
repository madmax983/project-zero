import re

with open('src/simulation.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''    if !world.contains_resource::<Events<HostileSpawnEvent>>() {
        world.init_resource::<Events<HostileSpawnEvent>>();
    }''',
'''    if !world.contains_resource::<Events<HostileSpawnEvent>>() {
        world.init_resource::<Events<HostileSpawnEvent>>();
    }
    if !world.contains_resource::<Events<crate::layer1::gravitational_debt::DebtReleaseEvent>>() {
        world.init_resource::<Events<crate::layer1::gravitational_debt::DebtReleaseEvent>>();
    }'''
)

with open('src/simulation.rs', 'w') as f:
    f.write(content)
