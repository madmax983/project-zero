use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::layer1::energy::{Battery, PowerConsumer};
use crate::layer1::fire::Fire;
use crate::layer1::map::GridPosition;
use crate::layer1::temperature::TemperatureGrid;

#[derive(Component)]
pub struct PowerCable {
    pub capacity: f32,
    pub current_load: f32,
}

pub fn evaluate_grid_load_system(
    mut cables: Query<(&mut PowerCable, &GridPosition)>,
    consumers: Query<(&PowerConsumer, &GridPosition)>,
    mut batteries: Query<(&mut Battery, &GridPosition)>,
    existing_fires: Query<&GridPosition, With<Fire>>,
    mut temperature_grid: ResMut<TemperatureGrid>,
    mut commands: Commands,
) {
    // Group consumers by tile to find local demand
    let mut tile_demand = HashMap::new();
    for (consumer, pos) in consumers.iter() {
        if consumer.active {
            *tile_demand.entry((pos.x, pos.y)).or_insert(0.0) += consumer.demand;
        }
    }

    // Attempt to buffer surges using local batteries
    let mut buffered_demand = HashMap::new();
    for (pos, demand) in tile_demand.iter() {
        let mut remaining_demand = *demand;
        // Find batteries on this tile
        for (mut battery, bat_pos) in batteries.iter_mut() {
            if bat_pos.x == pos.0 && bat_pos.y == pos.1 {
                let drained = battery.discharge(remaining_demand);
                remaining_demand -= drained;
                if remaining_demand <= 0.0 {
                    break;
                }
            }
        }
        buffered_demand.insert(*pos, remaining_demand);
    }

    // Check each cable
    for (mut cable, pos) in cables.iter_mut() {
        let demand = *buffered_demand.get(&(pos.x, pos.y)).unwrap_or(&0.0);
        cable.current_load = demand;

        if demand > cable.capacity {
            // Overloaded
            // Heat generation scales with overload amount
            let overload_amount = demand - cable.capacity;
            temperature_grid.add(pos.x, pos.y, overload_amount * 0.1); // Add some heat

            if demand > cable.capacity * 2.0 {
                // Extreme overload -> Fire
                // Prevent spawning dozens of fires per frame on the same tile
                let has_fire = existing_fires.iter().any(|p| p.x == pos.x && p.y == pos.y);
                if !has_fire {
                    commands.spawn((Fire::default(), *pos));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::energy::{PowerConsumer, Battery};
    use crate::layer1::temperature::TemperatureGrid;
    use crate::layer1::fire::Fire;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_cable_overload_creates_heat() {
        // Arrange
        let mut world = World::new();

        // Spawn a cable with max capacity 100
        let cable_pos = GridPosition { x: 5, y: 5 };
        let cable_entity = world.spawn((
            PowerCable { capacity: 100.0, current_load: 0.0 },
            cable_pos,
        )).id();

        // Connect a consumer drawing 150 power
        world.spawn((
            PowerConsumer { demand: 150.0, active: true },
            cable_pos,
        ));

        // Add a temperature grid
        world.insert_resource(TemperatureGrid::new(10, 10, 0.0));

        // Act: Evaluate grid
        let _ = world.run_system_once(evaluate_grid_load_system);

        // Assert: Cable is overloaded
        let cable = world.get::<PowerCable>(cable_entity).unwrap();
        assert!(cable.current_load > cable.capacity, "Cable load should exceed capacity");

        // Assert: Heat is generated on the tile
        let temperature_grid = world.resource::<TemperatureGrid>();
        let tile_heat = temperature_grid.get(cable_pos.x as usize, cable_pos.y as usize);
        assert!(tile_heat > 0.0, "Overloaded cable must generate heat");
    }

    #[test]
    fn test_extreme_overload_causes_fire() {
        let mut world = World::new();

        let pos = GridPosition { x: 3, y: 3 };
        let _cable = world.spawn((
            PowerCable { capacity: 50.0, current_load: 0.0 },
            pos,
        )).id();

        // Draw 300 power (massive overload)
        world.spawn((
            PowerConsumer { demand: 300.0, active: true },
            pos,
        ));

        world.insert_resource(TemperatureGrid::new(10, 10, 0.0));

        let _ = world.run_system_once(evaluate_grid_load_system);

        // Assert: Fire entity spawned on the cable position
        let mut fire_query = world.query_filtered::<&GridPosition, With<Fire>>();
        let fire_exists = fire_query.iter(&world).any(|p| p.x == 3 && p.y == 3);
        assert!(fire_exists, "Massive overload must start a fire");
    }

    #[test]
    fn test_batteries_buffer_surges() {
        let mut world = World::new();

        let pos = GridPosition { x: 2, y: 2 };
        world.spawn((
            PowerCable { capacity: 100.0, current_load: 0.0 },
            pos,
        ));

        // Add a battery that can absorb the surge
        world.spawn((
            Battery { capacity: 500.0, charge: 500.0, max_throughput: 100.0 },
            pos,
        ));

        world.spawn((
            PowerConsumer { demand: 150.0, active: true },
            pos,
        ));

        world.insert_resource(TemperatureGrid::new(10, 10, 0.0));

        let _ = world.run_system_once(evaluate_grid_load_system);

        // Assert: Cable is not overloaded because battery handles the local draw
        let temperature_grid = world.resource::<TemperatureGrid>();
        let tile_heat = temperature_grid.get(pos.x as usize, pos.y as usize);
        assert_eq!(tile_heat, 0.0, "Battery should buffer surge and prevent heat");
    }
}