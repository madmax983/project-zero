//! Swarm Intelligence and Hive Mind mechanics for Drones.
//!
//! Drones are usually simple-minded workers, but when grouped together in sufficient numbers,
//! they form a localized mesh network, upgrading their capabilities from `Low` to `High` intelligence.
//! This module handles the spatial clustering logic that detects when a "Swarm" has formed.

use crate::layer1::entities::drone::Drone;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

/// Defines the cognitive capacity of a drone based on network density.
///
/// # Examples
///
/// ```
/// use scale::layer1::entities::swarm_intelligence::IntelligenceLevel;
///
/// let isolated_drone = IntelligenceLevel::Low;
/// let swarm_drone = IntelligenceLevel::High;
/// assert_ne!(isolated_drone, swarm_drone);
/// ```
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum IntelligenceLevel {
    /// Drone operates on basic, isolated logic.
    Low,
    /// Drone is part of a swarm and executes advanced parallel tasks.
    High,
}

/// Component that tracks the current emergent behavior state of a drone.
///
/// By default, newly spawned drones have [`IntelligenceLevel::Low`].
///
/// # Examples
///
/// ```
/// use scale::layer1::entities::swarm_intelligence::{DroneBehavior, IntelligenceLevel};
///
/// let behavior = DroneBehavior::default();
/// assert_eq!(behavior.intelligence_level, IntelligenceLevel::Low);
/// ```
#[derive(Component)]
pub struct DroneBehavior {
    /// The current intelligence level, updated dynamically by clustering systems.
    pub intelligence_level: IntelligenceLevel,
}

impl Default for DroneBehavior {
    fn default() -> Self {
        Self {
            intelligence_level: IntelligenceLevel::Low,
        }
    }
}

/// System that upgrades or downgrades drone intelligence based on proximity to peers.
///
/// A drone requires at least 2 other drones within a `clustering_radius` of `5.0` tiles
/// to form a mesh network and achieve [`IntelligenceLevel::High`].
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::entities::drone::Drone;
/// use scale::layer1::entities::swarm_intelligence::{DroneBehavior, IntelligenceLevel, update_drone_clusters};
/// use scale::layer1::map::GridPosition;
///
/// let mut world = World::new();
///
/// // Spawn a cluster of 3 drones close together
/// world.spawn((Drone::default(), GridPosition { x: 0, y: 0 }, DroneBehavior::default()));
/// world.spawn((Drone::default(), GridPosition { x: 1, y: 0 }, DroneBehavior::default()));
/// world.spawn((Drone::default(), GridPosition { x: 0, y: 1 }, DroneBehavior::default()));
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(update_drone_clusters);
/// schedule.run(&mut world);
///
/// // All three drones should now have High intelligence
/// let mut query = world.query::<&DroneBehavior>();
/// for behavior in query.iter(&world) {
///     assert_eq!(behavior.intelligence_level, IntelligenceLevel::High);
/// }
/// ```
/// ⚡ Bolt Optimization:
/// - Replaced the un-sized `Vec::new()` allocation with `Vec::with_capacity(...)` based on the query size.
/// - This avoids repeated small memory allocations during the loop execution.
pub fn update_drone_clusters(
    mut query: Query<(Entity, &GridPosition, &mut DroneBehavior), With<Drone>>,
) {
    let mut drone_positions = Vec::with_capacity(query.iter().len());
    for (entity, pos, _) in query.iter() {
        drone_positions.push((entity, *pos));
    }

    let clustering_radius = 5.0;

    for (entity, pos, mut behavior) in query.iter_mut() {
        let nearby_count = drone_positions
            .iter()
            .filter(|(e, p)| {
                if *e == entity {
                    return false;
                }
                let dx = p.x as f32 - pos.x as f32;
                let dy = p.y as f32 - pos.y as f32;
                let distance = (dx * dx + dy * dy).sqrt();
                distance < clustering_radius
            })
            .count();

        if nearby_count >= 2 {
            behavior.intelligence_level = IntelligenceLevel::High;
        } else {
            behavior.intelligence_level = IntelligenceLevel::Low;
        }
    }
}

/// Executes drone tasks, scaling complexity based on current intelligence.
///
/// Drones with [`IntelligenceLevel::High`] can perform complex swarm logic,
/// while those with [`IntelligenceLevel::Low`] fall back to basic, "dumb" logic.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::entities::drone::Drone;
/// use scale::layer1::entities::swarm_intelligence::{DroneBehavior, IntelligenceLevel, update_drone_behavior};
///
/// let mut world = World::new();
/// world.spawn((
///     Drone::default(),
///     DroneBehavior { intelligence_level: IntelligenceLevel::High }
/// ));
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(update_drone_behavior);
/// schedule.run(&mut world);
/// // High intelligence logic executes cleanly without panic.
/// ```
pub fn update_drone_behavior(
    mut query: Query<(&DroneBehavior, &mut crate::layer1::utility_ai::PopAction), With<Drone>>,
) {
    for (behavior, mut action) in &mut query {
        if action.current == crate::layer1::utility_ai::ActionType::Charge {
            continue;
        }

        match behavior.intelligence_level {
            IntelligenceLevel::High => {
                if action.current == crate::layer1::utility_ai::ActionType::Idle
                    || action.current == crate::layer1::utility_ai::ActionType::Explore
                {
                    action.current = crate::layer1::utility_ai::ActionType::Repair;
                    action.current_utility = 0.9;
                }
            }
            IntelligenceLevel::Low => {
                if action.current == crate::layer1::utility_ai::ActionType::Idle
                    || action.current == crate::layer1::utility_ai::ActionType::Repair
                {
                    action.current = crate::layer1::utility_ai::ActionType::Explore;
                    action.current_utility = 0.8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_swarm_intelligence_basic_behavior() {
        let mut app = App::new();
        app.add_systems(Update, (update_drone_clusters, update_drone_behavior));

        let drone1 = app
            .world_mut()
            .spawn((
                Drone::default(),
                GridPosition { x: 0, y: 0 },
                DroneBehavior::default(),
            ))
            .id();
        let _drone2 = app
            .world_mut()
            .spawn((
                Drone::default(),
                GridPosition { x: 1, y: 0 },
                DroneBehavior::default(),
            ))
            .id();
        let _drone3 = app
            .world_mut()
            .spawn((
                Drone::default(),
                GridPosition { x: 0, y: 1 },
                DroneBehavior::default(),
            ))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .get::<DroneBehavior>(drone1)
                .unwrap()
                .intelligence_level,
            IntelligenceLevel::High
        );
    }

    #[test]
    fn test_swarm_intelligence_edge_cases() {
        let mut app = App::new();
        app.add_systems(Update, (update_drone_clusters, update_drone_behavior));

        let drone = app
            .world_mut()
            .spawn((
                Drone::default(),
                GridPosition { x: 100, y: 100 },
                DroneBehavior::default(),
            ))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .get::<DroneBehavior>(drone)
                .unwrap()
                .intelligence_level,
            IntelligenceLevel::Low
        );
    }
}
