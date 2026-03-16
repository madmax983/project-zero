import re

with open('src/layer1/systems/environment.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''    schedule.add_systems(
        (
            crate::layer1::shadow_market::despawn_in_light_system,
            crate::layer1::shadow_market::spawn_shadow_trader_system,
        )
            .in_set(Layer1SystemSet::Environment),
    );''',
'''    schedule.add_systems(
        (
            crate::layer1::shadow_market::despawn_in_light_system,
            crate::layer1::shadow_market::spawn_shadow_trader_system,
            crate::layer1::gravitational_debt::gravitational_debt_accumulation_system,
            crate::layer1::gravitational_debt::gravitational_debt_release_system,
            crate::layer1::gravitational_debt::process_gravitational_debt_release_system
                .after(crate::layer1::gravitational_debt::gravitational_debt_release_system),
        )
            .in_set(Layer1SystemSet::Environment),
    );'''
)

with open('src/layer1/systems/environment.rs', 'w') as f:
    f.write(content)
