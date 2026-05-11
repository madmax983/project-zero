use scale::layer1::atmosphere::{apply_smog_damage_system, AtmosphereGrid};
use scale::layer1::biology::health::Health;
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

#[test]
fn test_rust_lung_provides_smog_immunity() {
    let mut world = World::new();

    let pop_immune = world
        .spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: true,
            },
            GridPosition { x: 10, y: 10 },
        ))
        .id();

    let pop_vulnerable = world
        .spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            GridPosition { x: 10, y: 10 },
        ))
        .id();

    let mut grid = AtmosphereGrid::new(20, 20);
    grid.set(10, 10, 200.0);
    world.insert_resource(grid);

    world.run_system_once(apply_smog_damage_system).unwrap();

    let health_immune = world.get::<Health>(pop_immune).unwrap();
    assert_eq!(health_immune.current, 100.0, "Pop with Rust-Lung should be immune to smog");

    let health_vulnerable = world.get::<Health>(pop_vulnerable).unwrap();
    assert!(health_vulnerable.current < 100.0, "Pop without Rust-Lung should take smog damage");
}
