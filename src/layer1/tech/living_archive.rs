use bevy_ecs::prelude::*;
use crate::layer1::health::Health;

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

use crate::layer1::map::GridPosition;
use crate::layer1::nature::temperature::TemperatureGrid;

pub fn process_flesh_server_stress_system(
    servers: Query<(Entity, &Health, &GridPosition), With<FleshServer>>,
    mut blueprints: Query<(&mut Blueprint, &StoredIn)>,
    temp_grid: Option<Res<TemperatureGrid>>,
) {
    let mut is_stressed_cache = bevy_utils::HashMap::new();

    for (server_entity, health, pos) in servers.iter() {
        let mut temp_diff = 0.0;
        if let Some(ref grid) = temp_grid {
            let temp = grid.get(pos.x as usize, pos.y as usize);
            // Default target temperature for FleshServer is around 20.0 C
            let target_temp = 20.0;
            temp_diff = (temp - target_temp).abs();
        }

        let is_stressed = health.current < health.max * 0.8 || temp_diff > 10.0;
        is_stressed_cache.insert(server_entity, is_stressed);
    }

    for (mut blueprint, stored_in) in blueprints.iter_mut() {
        if let Some(&is_stressed) = is_stressed_cache.get(&stored_in.server) {
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
    use bevy_ecs::prelude::*;
    use crate::layer1::architecture::building::Building;
    use crate::layer1::architecture::building::BuildingType;
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::nature::temperature::TemperatureGrid;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_flesh_server_corrupts_data_when_stressed_health() {
        let mut world = setup_world();

        let server_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::ServerBank,
                },
                FleshServer,
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                }, // Damaged/Stressed (< 80%)
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let blueprint_entity = world
            .spawn((
                Blueprint {
                    base_cost: 100.0,
                    is_corrupted: false,
                },
                StoredIn {
                    server: server_entity,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_flesh_server_stress_system);
        schedule.run(&mut world);

        let blueprint = world.get::<Blueprint>(blueprint_entity).unwrap();
        assert!(blueprint.is_corrupted);
        assert_eq!(blueprint.base_cost, 200.0); // Cost doubled
    }

    #[test]
    fn test_flesh_server_corrupts_data_when_stressed_temperature() {
        let mut world = setup_world();

        let mut temp_grid = TemperatureGrid::new(10, 10, 0.0);
        temp_grid.set(5, 5, 40.0); // 40.0 C is > 20.0 C + 10.0 C tolerance
        world.insert_resource(temp_grid);

        let server_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::ServerBank,
                },
                FleshServer,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                }, // Healthy
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let blueprint_entity = world
            .spawn((
                Blueprint {
                    base_cost: 100.0,
                    is_corrupted: false,
                },
                StoredIn {
                    server: server_entity,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_flesh_server_stress_system);
        schedule.run(&mut world);

        let blueprint = world.get::<Blueprint>(blueprint_entity).unwrap();
        assert!(blueprint.is_corrupted);
        assert_eq!(blueprint.base_cost, 200.0); // Cost doubled
    }

    #[test]
    fn test_flesh_server_repairs_data_when_not_stressed() {
        let mut world = setup_world();

        let server_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::ServerBank,
                },
                FleshServer,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                }, // Healthy
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let blueprint_entity = world
            .spawn((
                Blueprint {
                    base_cost: 200.0,
                    is_corrupted: true, // Initially corrupted
                },
                StoredIn {
                    server: server_entity,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_flesh_server_stress_system);
        schedule.run(&mut world);

        let blueprint = world.get::<Blueprint>(blueprint_entity).unwrap();
        assert!(!blueprint.is_corrupted); // Fixed
        assert_eq!(blueprint.base_cost, 100.0); // Cost halved
    }
}
