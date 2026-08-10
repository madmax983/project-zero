use bevy::prelude::*;
use scale::layer1::core::integration::gravity_plating_power_bridge_system;
use scale::layer1::energy::PowerConsumer;
use scale::layer1::physics::gravity_plating::{
    apply_zero_g_movement_system, monitor_gravity_generator_power_system,
};
use scale::layer1::physics::gravity_plating::{
    CurrentZone, GravityGenerator, GravityState, MovementType, PowerGridEvent, TraitList, Zone,
};
use scale::layer1::pop::Pop;

#[test]
fn test_gravity_plating_failure_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<bevy_ecs::event::Events<PowerGridEvent>>();

    app.add_systems(
        Update,
        (
            gravity_plating_power_bridge_system,
            monitor_gravity_generator_power_system.after(gravity_plating_power_bridge_system),
            apply_zero_g_movement_system.after(monitor_gravity_generator_power_system),
        ),
    );

    let zone = app
        .world_mut()
        .spawn((Zone { id: 1 }, GravityState::Normal))
        .id();

    let generator = app
        .world_mut()
        .spawn((
            PowerConsumer {
                demand: 10.0,
                active: true,
            },
            GravityGenerator { target_zone: zone },
        ))
        .id();

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            MovementType::Walking,
            scale::layer1::physics::gravity_plating::Velocity { x: 0.0, y: 0.0, accum_x: 0.0, accum_y: 0.0 },
            CurrentZone { zone },
            TraitList { traits: vec![] },
        ))
        .id();

    // Initial run to clear the "Changed" flag on newly spawned PowerConsumer
    app.update();

    // Trigger failure by turning off active state
    app.world_mut()
        .get_mut::<PowerConsumer>(generator)
        .unwrap()
        .active = false;

    // The bridge should detect the change and emit NodeFailed
    app.update();

    let gravity = app.world().get::<GravityState>(zone).unwrap();
    assert_eq!(*gravity, GravityState::ZeroG);

    let move_type = app.world().get::<MovementType>(pop).unwrap();
    assert_eq!(*move_type, MovementType::Drifting);
}
