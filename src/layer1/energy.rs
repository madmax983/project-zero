//! Energy system for the colony.
//!
//! Handles power generation, distribution, and consumption.
//! Grids are formed dynamically based on connectivity via `Conduit`s and power-related buildings.

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use std::collections::{HashMap, HashSet, VecDeque};

/// Emits power to the grid.
#[derive(Component, Debug, Clone)]
pub struct PowerSource {
    /// Amount of power produced per tick.
    pub output: f32,
}

/// Consumes power from the grid.
#[derive(Component, Debug, Clone)]
pub struct PowerConsumer {
    /// Amount of power consumed per tick.
    pub demand: f32,
    /// Whether the consumer is currently powered.
    pub active: bool,
}

/// Connects power grid elements.
#[derive(Component, Debug, Clone)]
pub struct Conduit;

fn build_grid_map(world: &mut World) -> HashMap<(i32, i32), Entity> {
    let mut grid_map = HashMap::new();
    let mut query = world.query_filtered::<(
        Entity,
        &GridPosition,
    ), Or<(With<PowerSource>, With<PowerConsumer>, With<Conduit>)>>();

    for (entity, pos) in query.iter(world) {
        grid_map.insert((pos.x, pos.y), entity);
    }
    grid_map
}

fn bfs_grid(
    start_pos: (i32, i32),
    grid_map: &HashMap<(i32, i32), Entity>,
    world: &World,
    visited: &mut HashSet<(i32, i32)>,
) -> (f32, f32, Vec<Entity>) {
    let mut grid_entities = Vec::new();
    let mut queue = VecDeque::new();

    if visited.insert(start_pos) {
        queue.push_back(start_pos);
    }

    let mut total_production = 0.0;
    let mut total_demand = 0.0;

    while let Some(pos) = queue.pop_front() {
        if let Some(&entity) = grid_map.get(&pos) {
            grid_entities.push(entity);

            if let Some(source) = world.get::<PowerSource>(entity) {
                total_production += source.output;
            }
            if let Some(consumer) = world.get::<PowerConsumer>(entity) {
                total_demand += consumer.demand;
            }

            let neighbors = [
                (pos.0 + 1, pos.1),
                (pos.0 - 1, pos.1),
                (pos.0, pos.1 + 1),
                (pos.0, pos.1 - 1),
            ];

            for neighbor in neighbors {
                if grid_map.contains_key(&neighbor) && !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
    }
    (total_production, total_demand, grid_entities)
}

/// Calculates total production and demand for the grid connected to `start_entity`.
pub fn calculate_grid_stats(world: &mut World, start_entity: Entity) -> (f32, f32) {
    let grid_map = build_grid_map(world);
    let start_pos = match world.get::<GridPosition>(start_entity) {
        Some(p) => (p.x, p.y),
        None => return (0.0, 0.0),
    };

    let mut visited = HashSet::new();
    let (prod, demand, _) = bfs_grid(start_pos, &grid_map, world, &mut visited);
    (prod, demand)
}

/// System to update power grids.
/// Identifies connected components, sums production/demand, and enables/disables consumers.
pub fn power_grid_system(world: &mut World) {
    // 1. Build grid map
    let grid_map = build_grid_map(world);

    // 2. Find connected components
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let positions: Vec<(i32, i32)> = grid_map.keys().copied().collect();

    for start_pos in positions {
        if visited.contains(&start_pos) {
            continue;
        }

        // BFS for this grid
        let (total_production, total_demand, grid_entities) = bfs_grid(start_pos, &grid_map, world, &mut visited);

        // 3. Update consumers
        // MVP Rule: If Production >= Demand, all Active. Else, all Inactive.
        let active = total_production >= total_demand;

        for entity in grid_entities {
            if let Some(mut consumer) = world.get_mut::<PowerConsumer>(entity) {
                consumer.active = active;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PowerSource, PowerConsumer, Conduit, calculate_grid_stats};
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_power_components() {
        let source = PowerSource { output: 10.0 };
        let consumer = PowerConsumer { demand: 5.0, active: true };
        let _conduit = Conduit; // Marker

        assert_eq!(source.output, 10.0);
        assert_eq!(consumer.demand, 5.0);
    }

    #[test]
    fn test_grid_connectivity_isolated() {
        // Source and Consumer far apart, no conduit
        let mut world = World::new();

        // Generator at 0,0
        let generator = world.spawn((
            PowerSource { output: 10.0 },
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Generator }, // Assume new type
        )).id();

        // Consumer at 10,10
        let cons = world.spawn((
            PowerConsumer { demand: 5.0, active: false },
            GridPosition { x: 10, y: 10 },
            Building { building_type: BuildingType::Smelter },
        )).id();

        // Run calculation
        let (production, _demand) = calculate_grid_stats(&mut world, generator); // Pass root entity to flood fill

        // Gen is its own grid
        assert_eq!(production, 10.0);

        // Consumer is isolated
        let (c_prod, c_demand) = calculate_grid_stats(&mut world, cons);
        assert_eq!(c_prod, 0.0);
        assert_eq!(c_demand, 5.0);
    }

    #[test]
    fn test_grid_connectivity_connected() {
        let mut world = World::new();

        // Generator at 0,0
        let generator = world.spawn((
            PowerSource { output: 10.0 },
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Generator },
        )).id();

        // Conduit at 0,1
        world.spawn((
            Conduit,
            GridPosition { x: 0, y: 1 },
            Building { building_type: BuildingType::PowerPole }, // New type
        ));

        // Consumer at 0,2
        let _cons = world.spawn((
            PowerConsumer { demand: 5.0, active: false },
            GridPosition { x: 0, y: 2 },
            Building { building_type: BuildingType::Smelter },
        )).id();

        // Run calculation (start from generator)
        let (production, demand) = calculate_grid_stats(&mut world, generator);

        assert_eq!(production, 10.0);
        assert_eq!(demand, 5.0);
    }

    #[test]
    fn test_overload_shutdown() {
        // 10 Prod, 15 Demand
        let mut world = World::new();

        // Gen 10
        world.spawn((
            PowerSource { output: 10.0 },
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Generator },
        ));

        // Cons 1 (10)
        let c1 = world.spawn((
            PowerConsumer { demand: 10.0, active: true },
            GridPosition { x: 0, y: 1 },
            Conduit, // Connect them implicitly or explicitly
            Building { building_type: BuildingType::Smelter },
        )).id();

        // Cons 2 (5)
        let c2 = world.spawn((
            PowerConsumer { demand: 5.0, active: true },
            GridPosition { x: 0, y: 2 },
            Conduit,
            Building { building_type: BuildingType::Smelter },
        )).id();

        // Run system
        super::power_grid_system(&mut world);

        // Verify active state toggles
        // For MVP: If Demand > Production, ALL consumers on that grid shut down (active = false).
        // Since 15 > 10, both should be inactive.
        let c1_state = world.get::<PowerConsumer>(c1).unwrap();
        let c2_state = world.get::<PowerConsumer>(c2).unwrap();

        assert!(!c1_state.active);
        assert!(!c2_state.active);
    }
}
