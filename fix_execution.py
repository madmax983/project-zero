with open('src/layer1/systems/execution.rs', 'r') as f:
    content = f.read()

target = """    schedule.add_systems(
        (
            movement_system
                .after(apply_quirk_modifiers_system)
                .after(crate::layer1::fauna::fauna_behavior_system)
                .after(crate::layer1::physics::hit_stop::hit_stop_system),"""

replacement = """    schedule.add_systems(
        (
            crate::layer1::physics::gravity_plating::monitor_gravity_generator_power_system,
            crate::layer1::physics::gravity_plating::apply_zero_g_movement_system
                .after(crate::layer1::physics::gravity_plating::monitor_gravity_generator_power_system),
        )
            .in_set(Layer1SystemSet::Execution),
    );

    schedule.add_systems(
        (
            movement_system
                .after(apply_quirk_modifiers_system)
                .after(crate::layer1::fauna::fauna_behavior_system)
                .after(crate::layer1::physics::hit_stop::hit_stop_system)
                .after(crate::layer1::physics::gravity_plating::apply_zero_g_movement_system),"""

content = content.replace(target, replacement)

with open('src/layer1/systems/execution.rs', 'w') as f:
    f.write(content)
