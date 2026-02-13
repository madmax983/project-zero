use crate::layer1::building::{Building, BuildingType};
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::energy::PowerSource;
use crate::layer1::pop::{Pop, Speed};
use crate::layer1::quirks::{PlanetaryTrait, PlanetaryTraits, apply_quirk_modifiers_system};
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(PlanetaryTraits::default());
    world.insert_resource(DayNightCycle::default());
    world
}

#[test]
fn test_traits_initialization() {
    let world = setup_world();
    let traits = world.resource::<PlanetaryTraits>();
    assert!(traits.0.is_empty()); // Default is Earth-like (no quirks)
}

#[test]
fn test_high_gravity_slows_movement() {
    let mut world = setup_world();
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::HighGravity]));

    // Spawn pop with default speed
    let pop = world
        .spawn((
            Pop,
            Speed {
                base: 1.0,
                current: 1.0,
                accumulator: 0.0,
            },
        ))
        .id();

    // Run modifier system
    world.run_system_once(apply_quirk_modifiers_system).unwrap();

    // Check speed
    let speed = world.get::<Speed>(pop).unwrap();
    // High Gravity = 0.8x speed
    assert!(
        (speed.current - 0.8).abs() < f32::EPSILON,
        "Expected 0.8, got {}",
        speed.current
    );
}

#[test]
fn test_low_gravity_speeds_movement() {
    let mut world = setup_world();
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::LowGravity]));

    let pop = world
        .spawn((
            Pop,
            Speed {
                base: 1.0,
                current: 1.0,
                accumulator: 0.0,
            },
        ))
        .id();

    world.run_system_once(apply_quirk_modifiers_system).unwrap();

    let speed = world.get::<Speed>(pop).unwrap();
    // Low Gravity = 1.2x speed
    assert!(
        (speed.current - 1.2).abs() < f32::EPSILON,
        "Expected 1.2, got {}",
        speed.current
    );
}

#[test]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn test_rapid_orbit_shortens_day() {
    let mut world = setup_world();
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::RapidOrbit]));

    // Setup DayNightCycle with default
    let default_ticks = 250;
    world.insert_resource(DayNightCycle {
        ticks_per_day: default_ticks,
        ..Default::default()
    });

    // Run modifier system
    world.run_system_once(apply_quirk_modifiers_system).unwrap();

    let cycle = world.resource::<DayNightCycle>();
    // Rapid Orbit = 0.5x day length
    assert_eq!(cycle.ticks_per_day, (default_ticks as f32 * 0.5) as u64);
}

#[test]
fn test_dense_atmosphere_reduces_solar_power() {
    let mut world = setup_world();
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::DenseAtmosphere]));

    // Spawn Generator (Solar/Wind implied for this test context, or generic "Generator")
    // If 042 uses "Generator" for all power, we assume atmosphere affects efficiency generally
    // or specifically adds a penalty.
    let generator = world
        .spawn((
            Building {
                building_type: BuildingType::Generator,
            },
            PowerSource { output: 10.0 },
        ))
        .id();

    world.run_system_once(apply_quirk_modifiers_system).unwrap();

    let source = world.get::<PowerSource>(generator).unwrap();
    // Dense Atmosphere = 0.8x output
    assert!(
        (source.output - 8.0).abs() < f32::EPSILON,
        "Expected 8.0, got {}",
        source.output
    );
}

#[test]
fn test_modifiers_stack() {
    let mut world = setup_world();
    // High Gravity (0.8 speed) + Low Gravity (1.2 speed) = 0.96 speed
    world.insert_resource(PlanetaryTraits(vec![
        PlanetaryTrait::HighGravity,
        PlanetaryTrait::LowGravity,
    ]));

    let pop = world
        .spawn((
            Pop,
            Speed {
                base: 1.0,
                current: 1.0,
                accumulator: 0.0,
            },
        ))
        .id();

    world.run_system_once(apply_quirk_modifiers_system).unwrap();

    let speed = world.get::<Speed>(pop).unwrap();
    assert!(
        (speed.current - 0.96).abs() < f32::EPSILON,
        "Expected 0.96, got {}",
        speed.current
    );
}
