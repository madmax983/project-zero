use bevy::prelude::*;
use scale::layer1::physics::gravity_plating::{GravityState, MovementType, PowerGridEvent, Zone, GravityGenerator, PowerNode, CurrentZone, TraitList};
use scale::layer1::pop::Pop;
use scale::layer1::physics::gravity_plating::{monitor_gravity_generator_power_system, apply_zero_g_movement_system};

#[test]
fn test_gravity_plating_failure_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Test the integration functionality in isolation to avoid massive App setup requirements
    // Ensure PowerGridEvent can trigger drifting mechanics
    app.init_resource::<bevy_ecs::event::Events<PowerGridEvent>>();

    app.add_systems(Update, (
        monitor_gravity_generator_power_system,
        apply_zero_g_movement_system.after(monitor_gravity_generator_power_system),
    ));

    let zone = app.world_mut().spawn((Zone { id: 1 }, GravityState::Normal)).id();

    let generator = app.world_mut().spawn((
        PowerNode { current_power: 0, required_power: 100 },
        GravityGenerator { target_zone: zone },
    )).id();

    let pop = app.world_mut().spawn((
        Pop,
        MovementType::Walking,
        scale::layer1::physics::gravity_plating::Velocity { x: 0.0, y: 0.0 },
        CurrentZone { zone },
        TraitList { traits: vec![] },
    )).id();

    // Trigger failure
    app.world_mut().resource_mut::<Events<PowerGridEvent>>().send(PowerGridEvent::NodeFailed(generator));

    app.update();

    let gravity = app.world().get::<GravityState>(zone).unwrap();
    assert_eq!(*gravity, GravityState::ZeroG);

    let move_type = app.world().get::<MovementType>(pop).unwrap();
    assert_eq!(*move_type, MovementType::Drifting);
}
