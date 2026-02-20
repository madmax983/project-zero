//! Bureaucratic Drag system (Spec 175).
//!
//! As the colony grows, administration slows down.
//! Efficiency = Supply (Offices/Admins) / Demand (Buildings/Pops).

use bevy_ecs::prelude::*;

/// Tracks global administrative efficiency.
#[derive(Resource, Default, Debug)]
pub struct AdminStats {
    /// Total Admin points provided by offices and administrators.
    pub supply: f32,
    /// Total Admin points consumed by buildings and pops.
    pub demand: f32,
    /// Global efficiency multiplier (0.0 to 1.0).
    /// Applied to Work Speed and Construction Speed.
    pub efficiency: f32,
}

/// Component for entities that provide Admin points.
#[derive(Component, Default, Debug)]
pub struct AdminProvider {
    /// Amount of Admin points provided.
    pub amount: f32,
}

/// Component for entities that consume Admin points.
#[derive(Component, Default, Debug)]
pub struct AdminConsumer {
    /// Amount of Admin points consumed.
    pub demand: f32,
}

/// Component for Office buildings.
#[derive(Component, Debug, Clone)]
pub struct Office {
    /// Max number of administrators.
    pub capacity: usize,
    /// Current administrators.
    pub workers: Vec<Entity>,
}

impl Default for Office {
    fn default() -> Self {
        Self {
            capacity: 2,
            workers: Vec::new(),
        }
    }
}

/// Calculates the global Admin efficiency.
pub fn calculate_admin_stats(world: &mut World) {
    let mut supply = 0.0;
    let mut demand = 0.0;

    // Sum Providers
    let mut providers = world.query::<&AdminProvider>();
    for provider in providers.iter(world) {
        supply += provider.amount;
    }

    // Sum Consumers
    let mut consumers = world.query::<&AdminConsumer>();
    for consumer in consumers.iter(world) {
        demand += consumer.demand;
    }

    // Default to 1.0 efficiency if no demand
    let efficiency = if demand <= f32::EPSILON {
        1.0
    } else {
        (supply / demand).clamp(0.0, 1.0)
    };

    world.insert_resource(AdminStats {
        supply,
        demand,
        efficiency,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::{Job, JobType, Pop};

    #[test]
    fn test_admin_stats_calculation() {
        let mut world = World::new();
        world.insert_resource(AdminStats::default());

        // 1. Spawn Provider (Office with Administrator)
        // We simulate Office by manually adding AdminProvider since we haven't updated BuildingType yet
        let office = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing, // Placeholder until Office exists
                },
                AdminProvider { amount: 10.0 }, // Base provided by building
            ))
            .id();

        // Spawn Administrator working there
        world.spawn((
            Pop,
            Job {
                workplace: office,
                job_type: JobType::Administrator,
            },
            AdminProvider { amount: 5.0 }, // Provided by the job
        ));

        // 2. Spawn Consumer (Housing)
        world.spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            AdminConsumer { demand: 2.0 },
        ));

        // 3. Spawn Consumer (Pop)
        world.spawn((
            Pop,
            AdminConsumer { demand: 1.0 },
        ));

        // Run calculation
        calculate_admin_stats(&mut world);

        let stats = world.resource::<AdminStats>();

        // Supply: 10 (Building) + 5 (Job) = 15
        assert_eq!(stats.supply, 15.0);

        // Demand: 2 (Building) + 1 (Pop) = 3
        assert_eq!(stats.demand, 3.0);

        // Efficiency: 15 / 3 = 5.0 -> Clamped to 1.0
        assert_eq!(stats.efficiency, 1.0);
    }

    #[test]
    fn test_admin_efficiency_penalty() {
        let mut world = World::new();
        world.insert_resource(AdminStats::default());

        // Supply: 5
        world.spawn(AdminProvider { amount: 5.0 });

        // Demand: 10
        world.spawn(AdminConsumer { demand: 10.0 });

        calculate_admin_stats(&mut world);

        let stats = world.resource::<AdminStats>();

        // Efficiency: 5 / 10 = 0.5
        assert_eq!(stats.efficiency, 0.5);
    }

    #[test]
    fn test_zero_demand_handling() {
        let mut world = World::new();
        world.insert_resource(AdminStats::default());

        // Supply: 5, Demand: 0
        world.spawn(AdminProvider { amount: 5.0 });

        calculate_admin_stats(&mut world);

        let stats = world.resource::<AdminStats>();
        assert_eq!(stats.efficiency, 1.0); // Should not be Infinity or NaN
    }
}
