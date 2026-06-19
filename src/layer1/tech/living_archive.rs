use crate::layer1::energy::EnergyGrid;
use crate::layer1::health::Health;
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

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
    servers: Query<(Entity, &Health, &EnergyGrid), With<FleshServer>>,
    mut blueprints: Query<(&mut Blueprint, &StoredIn)>,
) {
    for (server_entity, health, energy) in servers.iter() {
        // Stressed if health drops below 80% or if there is not enough power
        let is_stressed =
            health.current < health.max * 0.8 || energy.total_consumption > energy.total_generation;

        for (mut blueprint, stored_in) in blueprints.iter_mut() {
            if stored_in.server == server_entity {
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
}

pub fn flesh_server_nutrient_consumption_system(
    mut servers: Query<&mut Health, With<FleshServer>>,
    mut resources: ResMut<ColonyResources>,
) {
    for mut health in servers.iter_mut() {
        // Flesh servers consume 10.0 food per tick
        if resources.food >= 10.0 {
            resources.food -= 10.0;
        } else {
            // Take damage if starved
            resources.food = 0.0;
            health.current -= 5.0; // Damage per tick
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::Building;
    use crate::layer1::energy::EnergyGrid;
    use crate::layer1::health::Health;
    use crate::layer1::resources::ColonyResources;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_flesh_server_corrupts_data_when_stressed() {
        let mut world = setup_world();

        let server_entity = world
            .spawn((
                Building {
                    building_type: crate::layer1::building::BuildingType::Housing,
                }, // Using housing as a dummy since FleshServer building type might not exist yet
                FleshServer,
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                }, // Damaged/Stressed (Health < 80%)
                EnergyGrid {
                    total_generation: 50.0,
                    total_consumption: 100.0,
                }, // Overheating / Power Starved
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
    fn test_flesh_server_nutrient_consumption() {
        let mut world = setup_world();
        world.insert_resource(ColonyResources {
            food: 15.0,
            ..Default::default()
        });

        let server_entity = world
            .spawn((
                FleshServer,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(flesh_server_nutrient_consumption_system);

        // Tick 1: Consumes 10.0 food
        schedule.run(&mut world);
        assert_eq!(world.resource::<ColonyResources>().food, 5.0);
        assert_eq!(world.get::<Health>(server_entity).unwrap().current, 100.0);

        // Tick 2: Insufficient food (5.0 < 10.0), starved, takes 5.0 damage
        schedule.run(&mut world);
        assert_eq!(world.resource::<ColonyResources>().food, 0.0);
        assert_eq!(world.get::<Health>(server_entity).unwrap().current, 95.0);
    }
}
