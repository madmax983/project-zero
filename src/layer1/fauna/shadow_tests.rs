use bevy_ecs::prelude::*;
use crate::layer1::fauna::shadow::{ShadowEntity, ShadowType, spawn_shadow_fauna_system, shadow_visibility_system, DataDensityGrid, shadow_feed_system};
use crate::layer1::map::GridPosition;
use crate::layer1::energy::PowerSource;

#[test]
fn test_high_energy_spawns_static_mites() {
    let mut world = World::new();
    // Setup high energy grid cell
    world.spawn((
        GridPosition { x: 5, y: 5 },
        PowerSource {
            output: 1000.0, // Very high load
            active: true,
        }
    ));

    world.insert_resource(DataDensityGrid::new(10, 10));

    // Run spawn system
    let mut schedule = Schedule::default();
    schedule.add_systems(spawn_shadow_fauna_system);
    schedule.run(&mut world);

    // Check for entity
    let mut query = world.query::<(&ShadowEntity, &GridPosition)>();
    let found = query.iter(&world).any(|(e, pos)|
        e.entity_type == ShadowType::StaticMite && pos.x == 5 && pos.y == 5
    );
    assert!(found, "StaticMite should spawn on high power load");
}

#[test]
fn test_high_data_density_spawns_data_rot() {
    let mut world = World::new();
    let mut grid = DataDensityGrid::new(10, 10);
    grid.set(2, 2, 500.0); // High data density
    world.insert_resource(grid);

    let mut schedule = Schedule::default();
    schedule.add_systems(spawn_shadow_fauna_system);
    schedule.run(&mut world);

    let mut query = world.query::<(&ShadowEntity, &GridPosition)>();
    let found = query.iter(&world).any(|(e, pos)|
        e.entity_type == ShadowType::DataRot && pos.x == 2 && pos.y == 2
    );
    assert!(found, "DataRot should spawn on high data density");
}

#[test]
fn test_shadow_feed_reduces_power_efficiency() {
    let mut world = World::new();

    let generator = world.spawn((
        GridPosition { x: 1, y: 1 },
        PowerSource {
            output: 100.0,
            active: true,
        }
    )).id();

    world.spawn((
        GridPosition { x: 1, y: 1 },
        ShadowEntity {
            entity_type: ShadowType::StaticMite,
            hunger: 10.0,
            visible: false,
        }
    ));

    // This simulates a system that applies the efficiency penalty
    let mut schedule = Schedule::default();
    schedule.add_systems(shadow_feed_system);
    schedule.run(&mut world);

    let power_source = world.get::<PowerSource>(generator).unwrap();
    assert!(power_source.output < 100.0, "StaticMite should reduce local machine output");
}

#[test]
fn test_visibility_toggle() {
    let mut world = World::new();

    let shadow = world.spawn((
        GridPosition { x: 3, y: 3 },
        ShadowEntity {
            entity_type: ShadowType::VoidEel,
            hunger: 0.0,
            visible: false,
        }
    )).id();

    // Setup a state where they should be visible (e.g. Magnetic Storm active)
    world.insert_resource(crate::layer1::fauna::shadow::MagneticStorm { active: true });

    let mut schedule = Schedule::default();
    schedule.add_systems(shadow_visibility_system);
    schedule.run(&mut world);

    let shadow_comp = world.get::<ShadowEntity>(shadow).unwrap();
    assert!(shadow_comp.visible, "Shadow entity should be visible during Magnetic Storm");

    // Turn off storm
    world.insert_resource(crate::layer1::fauna::shadow::MagneticStorm { active: false });
    schedule.run(&mut world);

    let shadow_comp = world.get::<ShadowEntity>(shadow).unwrap();
    assert!(!shadow_comp.visible, "Shadow entity should be invisible when Magnetic Storm is inactive");
}
