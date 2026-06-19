// src/layer1/tech/living_archive.rs

use bevy_ecs::prelude::*;
use crate::layer1::biology::health::Health;
use crate::layer1::core::map::GridPosition;
use crate::layer1::nature::temperature::TemperatureGrid;

#[derive(Component)]
pub struct FleshServer;

#[derive(Component, Debug, Clone)]
pub struct Blueprint {
    pub base_cost: f32,
    pub is_corrupted: bool,
}

#[derive(Component)]
pub struct StoredIn {
    pub server: Entity,
}

pub fn process_flesh_server_stress_system(
    servers: Query<(&Health, &GridPosition), With<FleshServer>>,
    mut blueprints: Query<(&mut Blueprint, &StoredIn)>,
    temp_grid: Option<Res<TemperatureGrid>>,
) {
    for (mut blueprint, stored_in) in blueprints.iter_mut() {
        if let Ok((health, pos)) = servers.get(stored_in.server) {
            let temp = if let Some(grid) = &temp_grid {
                if pos.x < 0 || pos.y < 0 {
                    grid.ambient
                } else {
                    grid.get(pos.x as usize, pos.y as usize)
                }
            } else {
                20.0 // Default optimal temperature
            };

            // 20.0 is the optimal temperature in RED phase. Anything deviating > 10.0 causes stress
            let is_stressed = health.current < health.max * 0.8 || (temp - 20.0).abs() > 10.0;

            if is_stressed && !blueprint.is_corrupted {
                blueprint.is_corrupted = true;
                blueprint.base_cost *= 2.0;
            } else if !is_stressed && blueprint.is_corrupted {
                blueprint.is_corrupted = false;
                blueprint.base_cost /= 2.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::Building;
    use crate::layer1::biology::health::Health;
    use crate::layer1::nature::temperature::TemperatureGrid;
    use crate::layer1::core::map::GridPosition;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_flesh_server_corrupts_data_when_stressed() {
        let mut world = setup_world();

        let mut grid = TemperatureGrid::new(10, 10, 20.0);
        grid.set(5, 5, 40.0); // Overheating at position (5,5)
        world.insert_resource(grid);

        let server_entity = world.spawn((
            Building { building_type: crate::layer1::architecture::building::BuildingType::ServerBank },
            FleshServer,
            Health { current: 50.0, max: 100.0, has_rust_lung: false }, // Damaged/Stressed
            GridPosition { x: 5, y: 5 }, // Position matches overheated tile
        )).id();

        let blueprint_entity = world.spawn((
            Blueprint { base_cost: 100.0, is_corrupted: false },
            StoredIn { server: server_entity },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_flesh_server_stress_system);
        schedule.run(&mut world);

        let blueprint = world.get::<Blueprint>(blueprint_entity).unwrap();
        assert!(blueprint.is_corrupted);
        assert_eq!(blueprint.base_cost, 200.0); // Cost doubled
    }

    #[test]
    fn test_flesh_server_restores_data_when_healthy() {
        let mut world = setup_world();

        let mut grid = TemperatureGrid::new(10, 10, 20.0);
        grid.set(5, 5, 20.0); // Optimal temp
        world.insert_resource(grid);

        let server_entity = world.spawn((
            Building { building_type: crate::layer1::architecture::building::BuildingType::ServerBank },
            FleshServer,
            Health { current: 100.0, max: 100.0, has_rust_lung: false }, // Healthy
            GridPosition { x: 5, y: 5 },
        )).id();

        let blueprint_entity = world.spawn((
            Blueprint { base_cost: 200.0, is_corrupted: true },
            StoredIn { server: server_entity },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_flesh_server_stress_system);
        schedule.run(&mut world);

        let blueprint = world.get::<Blueprint>(blueprint_entity).unwrap();
        assert!(!blueprint.is_corrupted);
        assert_eq!(blueprint.base_cost, 100.0); // Cost restored to normal
    }

    #[test]
    fn test_flesh_server_fallback_when_no_temp_grid() {
        let mut world = setup_world();

        let server_entity = world.spawn((
            Building { building_type: crate::layer1::architecture::building::BuildingType::ServerBank },
            FleshServer,
            Health { current: 50.0, max: 100.0, has_rust_lung: false }, // Health Stressed
            GridPosition { x: 5, y: 5 },
        )).id();

        let blueprint_entity = world.spawn((
            Blueprint { base_cost: 100.0, is_corrupted: false },
            StoredIn { server: server_entity },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_flesh_server_stress_system);
        schedule.run(&mut world);

        let blueprint = world.get::<Blueprint>(blueprint_entity).unwrap();
        assert!(blueprint.is_corrupted);
        assert_eq!(blueprint.base_cost, 200.0);
    }
}