import sys

filename = 'src/setup.rs'
with open(filename, 'r') as file:
    content = file.read()

content = content.replace(
    'world.insert_resource(crate::layer1::social::golden_age::ColonySafety { days_without_incident: 0 });',
    'world.insert_resource(crate::layer1::social::golden_age::ColonySafety { days_without_incident: 0 });\n    world.init_resource::<bevy_ecs::event::Events<crate::layer1::social::golden_age::AlertEvent>>();'
)

with open(filename, 'w') as file:
    file.write(content)
