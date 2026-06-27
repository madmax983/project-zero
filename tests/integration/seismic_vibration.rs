use scale::layer1::building::BuildingType;
use scale::layer1::environment::seismic::VibrationGrid;
use scale::layer1::flora::Flora;
use scale::layer1::map::GridPosition;
use scale::setup::setup_world;
use scale::simulation::run_simulation_tick;

#[test]
fn test_generator_emits_vibration() {
    let mut world = setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::nature::ecology::HarvestEvent>>();

    // 1. Spawn a Generator at (10, 10)
    // We can use try_place_building or spawn_building directly.
    // Since we want to test the configuration logic (adding SeismicSource), we should use spawn_building
    // via try_place_building or just call spawn_building.
    // try_place_building requires resources, so let's use the helper spawn_building
    // but that's in building module and might not be public or easily accessible.
    // Actually, `scale::layer1::building::spawn_building` is public.

    scale::layer1::building::spawn_building(
        &mut world,
        10,
        10,
        BuildingType::Generator,
        scale::layer1::building::MaterialType::default(),
    );

    // 2. Run simulation
    run_simulation_tick(&mut world);

    // 3. Check VibrationGrid
    // Note: This will fail if VibrationGrid resource is missing (which it is currently)
    // It will panic at world.resource::<VibrationGrid>()

    // We expect this panic, but for the sake of the test being correct *eventually*,
    // we write it as if it works.

    if !world.contains_resource::<VibrationGrid>() {
        // Fail gracefully for now so we can see other failures?
        // No, let it panic, that's a good "Red" state.
        panic!("VibrationGrid resource missing!");
    }

    let grid = world.resource::<VibrationGrid>();
    let vibration = grid.get(10, 10);
    assert!(
        vibration > 0.0,
        "Generator should emit vibration at its location"
    );

    let vibration_near = grid.get(11, 10);
    assert!(
        vibration_near > 0.0,
        "Vibration should propagate to neighbors"
    );
}

#[test]
fn test_flora_agitation() {
    let mut world = setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::nature::ecology::HarvestEvent>>();

    // 1. Spawn a Generator
    scale::layer1::building::spawn_building(
        &mut world,
        20,
        20,
        BuildingType::Generator,
        scale::layer1::building::MaterialType::default(),
    );

    // 2. Spawn Flora nearby
    let flora_entity = world
        .spawn((
            Flora {
                growth_timer: 1000,
                attack_timer: 1000,
                ..Default::default()
            },
            GridPosition { x: 21, y: 20 },
        ))
        .id();

    // 3. Run simulation
    run_simulation_tick(&mut world);

    // 4. Check Flora timers
    if !world.contains_resource::<VibrationGrid>() {
        panic!("VibrationGrid resource missing!");
    }

    let flora = world.get::<Flora>(flora_entity).unwrap();
    // Normal tick reduction is 1 (or whatever the system does).
    // Agitation should reduce it by more.
    // If the system is running, vibration at (21, 20) should be > 0.
    // Let's assume update_seismic_system ran.

    // We can't easily know the exact reduction without knowing the vibration value,
    // but it should be < 999 (1 tick).
    // The seismic system reduces by `(vibration * 10.0) as u32`.
    // Generator intensity is 1.0. At distance 1, it's roughly 0.8 * transmission.
    // So reduction should be significant.

    assert!(
        flora.growth_timer < 999,
        "Flora growth timer should be reduced by vibration (was {}, expected < 999)",
        flora.growth_timer
    );
}
