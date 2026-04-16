use bevy_ecs::prelude::*;
use scale::layer1::day_night::{DayNightCycle, TimeOfDay};
use scale::layer1::energy::PowerSource;
use scale::layer1::environment::ephemeral_moons::{
    apply_moon_modifiers_system, EphemeralMoon, MoonType,
};
use scale::layer1::solar::SolarPower;

#[test]
fn test_bright_moon_boosts_solar_power_integration() {
    scale::setup::init_task_pools();
    let mut world = World::new();

    // 1. Initialize environment (Night Time)
    let env = DayNightCycle {
        time_of_day: TimeOfDay::Night,
        ..Default::default()
    };
    world.insert_resource(env);

    // 2. Spawn a Bright Moon
    world.spawn(EphemeralMoon {
        moon_type: MoonType::Bright,
        days_remaining: 10.0,
    });

    // 3. Spawn a Solar Panel
    let solar_panel = world
        .spawn((
            SolarPower { base_output: 10.0 },
            PowerSource {
                output: 0.0,
                active: true,
            },
        ))
        .id();

    // 4. Run the integration seam (apply modifiers)
    let mut schedule = Schedule::default();
    schedule.add_systems(apply_moon_modifiers_system);
    schedule.run(&mut world);

    // 5. Verify the seam is connected
    let source = world.get::<PowerSource>(solar_panel).unwrap();
    // Normally at night, output is 0. With a bright moon, it gets a 50% boost (+5.0)
    assert!(
        source.output > 0.0,
        "Solar output should be boosted by the bright moon during the night"
    );
}
