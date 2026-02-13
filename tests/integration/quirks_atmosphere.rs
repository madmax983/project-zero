use bevy_ecs::prelude::*;
use scale::layer1::atmosphere::{AtmosphereGrid, update_atmosphere_system};
use scale::layer1::quirks::{PlanetaryTrait, PlanetaryTraits, apply_quirk_modifiers_system};
use scale::layer1::pop::{Pop, Speed};
use scale::layer1::day_night::DayNightCycle; // Required by quirk system

// Helper to setup world with required resources for quirk system
fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(AtmosphereGrid::new(10, 10));
    world.insert_resource(PlanetaryTraits(vec![]));
    // Quirk system requires DayNightCycle
    world.insert_resource(DayNightCycle::default());

    // We also need to register the systems if we use schedule, but here we run them manually via run_system_once
    world
}

#[test]
fn test_dense_atmosphere_increases_pollution_retention() {
    let mut world = setup_world();

    // 1. Set Dense Atmosphere
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::DenseAtmosphere]));

    // 2. Set initial pollution
    let start_val = 10.0;
    world.resource_mut::<AtmosphereGrid>().set(5, 5, start_val);

    // 3. Run quirk system (should update diffusion rate)
    // Note: In current code this does nothing to AtmosphereGrid
    use bevy_ecs::system::RunSystemOnce;
    world.run_system_once(apply_quirk_modifiers_system).unwrap();

    // 4. Run atmosphere system (diffuse)
    // Note: Current diffuse() uses hardcoded 0.99
    world.run_system_once(update_atmosphere_system).unwrap();

    // 5. Check retention
    let grid = world.resource::<AtmosphereGrid>();
    let center_val = grid.get(5, 5);

    // Expected behavior:
    // Standard decay is 0.99.
    // Dense atmosphere should be higher (e.g. 0.999).
    // Center value after 1 tick of diffusion also depends on spread to neighbors.
    // Neighbors get 0.0 + spread. Center gets center - spread.
    // Then everything is multiplied by decay.

    // If we can inspect the grid struct, maybe we can check diffusion_rate directly?
    // But diffusion_rate doesn't exist yet on the struct.
    // So we must rely on behavior.

    // Let's compare against a control (normal atmosphere).
    let val_dense = center_val;

    // Control: Normal Atmosphere
    let mut world_normal = setup_world();
    world_normal.resource_mut::<AtmosphereGrid>().set(5, 5, start_val);
    world_normal.run_system_once(apply_quirk_modifiers_system).unwrap();
    world_normal.run_system_once(update_atmosphere_system).unwrap();
    let val_normal = world_normal.resource::<AtmosphereGrid>().get(5, 5);

    // Dense should retain MORE pollution (higher value) than Normal
    // Or if it traps it, maybe it spreads less?
    // "Dense Atmosphere" usually implies things stick around longer.
    // So retention (decay factor) should be closer to 1.0.

    assert!(val_dense > val_normal, "Dense atmosphere should retain more pollution (val_dense: {}, val_normal: {})", val_dense, val_normal);
}

#[test]
fn test_thin_atmosphere_decreases_pollution_retention() {
    let mut world = setup_world();

    // 1. Set Thin Atmosphere
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::ThinAtmosphere]));

    // 2. Set initial pollution
    let start_val = 10.0;
    world.resource_mut::<AtmosphereGrid>().set(5, 5, start_val);

    // 3. Run systems
    use bevy_ecs::system::RunSystemOnce;
    world.run_system_once(apply_quirk_modifiers_system).unwrap();
    world.run_system_once(update_atmosphere_system).unwrap();

    let val_thin = world.resource::<AtmosphereGrid>().get(5, 5);

    // Control: Normal Atmosphere
    let mut world_normal = setup_world();
    world_normal.resource_mut::<AtmosphereGrid>().set(5, 5, start_val);
    world_normal.run_system_once(apply_quirk_modifiers_system).unwrap();
    world_normal.run_system_once(update_atmosphere_system).unwrap();
    let val_normal = world_normal.resource::<AtmosphereGrid>().get(5, 5);

    // Thin should retain LESS pollution (lower value) than Normal
    assert!(val_thin < val_normal, "Thin atmosphere should clear pollution faster (val_thin: {}, val_normal: {})", val_thin, val_normal);
}

#[test]
fn test_high_gravity_slows_movement() {
    let mut world = setup_world();

    // 1. Set High Gravity
    world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::HighGravity]));

    // 2. Spawn Pop
    let pop = world.spawn((
        Pop,
        Speed { base: 1.0, current: 1.0, accumulator: 0.0 }
    )).id();

    // 3. Run quirk system
    use bevy_ecs::system::RunSystemOnce;
    world.run_system_once(apply_quirk_modifiers_system).unwrap();

    // 4. Verify Speed
    let speed = world.get::<Speed>(pop).unwrap();
    // HighGravity is 0.8 modifier
    assert!((speed.current - 0.8).abs() < f32::EPSILON, "High Gravity should slow speed to 0.8");
}
