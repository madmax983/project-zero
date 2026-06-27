use bevy_ecs::prelude::*;
use scale::layer1::atmosphere::{
    simulate_diffusion_system, update_atmosphere_system, AtmosphereGrid, DiffusionConfig,
};
use scale::layer1::day_night::DayNightCycle;
use scale::layer1::pop::{Pop, Speed};
use scale::layer1::quirks::{apply_quirk_modifiers_system, PlanetaryTrait, PlanetaryTraits};

fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(AtmosphereGrid::new(10, 10));
    world.insert_resource(PlanetaryTraits(vec![]));
    world.insert_resource(DayNightCycle::default());
    world.insert_resource(DiffusionConfig::default());
    world
}

#[test]
fn test_dense_atmosphere_increases_pollution_retention() {
    let mut world = setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::DenseAtmosphere]));

    let start_val = 10.0;
    world.resource_mut::<AtmosphereGrid>().set(5, 5, start_val);

    use bevy_ecs::system::RunSystemOnce;

    // Explicitly update AtmosphereGrid's diffusion_rate to be the one computed by apply_quirk_modifiers_system
    world.run_system_once(apply_quirk_modifiers_system).unwrap();
    // Simulate 10 ticks
    for _ in 0..10 {
        world.run_system_once(update_atmosphere_system).unwrap();
        world.run_system_once(simulate_diffusion_system).unwrap();
    }

    let val_dense = world.resource::<AtmosphereGrid>().get(5, 5);

    let mut world_normal = setup_world();
    world_normal
        .resource_mut::<AtmosphereGrid>()
        .set(5, 5, start_val);
    world_normal
        .run_system_once(apply_quirk_modifiers_system)
        .unwrap();

    for _ in 0..10 {
        world_normal
            .run_system_once(update_atmosphere_system)
            .unwrap();
        world_normal
            .run_system_once(simulate_diffusion_system)
            .unwrap();
    }

    let val_normal = world_normal.resource::<AtmosphereGrid>().get(5, 5);

    assert!(
        val_dense > val_normal,
        "Dense atmosphere should retain more pollution (val_dense: {}, val_normal: {})",
        val_dense,
        val_normal
    );
}

#[test]
fn test_thin_atmosphere_decreases_pollution_retention() {
    let mut world = setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::ThinAtmosphere]));

    let start_val = 10.0;
    world.resource_mut::<AtmosphereGrid>().set(5, 5, start_val);

    use bevy_ecs::system::RunSystemOnce;

    world.run_system_once(apply_quirk_modifiers_system).unwrap();
    for _ in 0..10 {
        world.run_system_once(update_atmosphere_system).unwrap();
        world.run_system_once(simulate_diffusion_system).unwrap();
    }

    let val_thin = world.resource::<AtmosphereGrid>().get(5, 5);

    let mut world_normal = setup_world();
    world_normal
        .resource_mut::<AtmosphereGrid>()
        .set(5, 5, start_val);
    world_normal
        .run_system_once(apply_quirk_modifiers_system)
        .unwrap();
    for _ in 0..10 {
        world_normal
            .run_system_once(update_atmosphere_system)
            .unwrap();
        world_normal
            .run_system_once(simulate_diffusion_system)
            .unwrap();
    }

    let val_normal = world_normal.resource::<AtmosphereGrid>().get(5, 5);

    assert!(
        val_thin < val_normal,
        "Thin atmosphere should clear pollution faster (val_thin: {}, val_normal: {})",
        val_thin,
        val_normal
    );
}

#[test]
fn test_high_gravity_slows_movement() {
    let mut world = setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::HighGravity]));

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

    use bevy_ecs::system::RunSystemOnce;
    world.run_system_once(apply_quirk_modifiers_system).unwrap();

    let speed = world.get::<Speed>(pop).unwrap();
    assert!(
        (speed.current - 0.8).abs() < f32::EPSILON,
        "High Gravity should slow speed to 0.8"
    );
}
