//! Drone entities and automation logic.
//!
//! This module defines the `Drone` entity and its associated states and systems.
//! Drones are automated workers that operate without the complex needs of normal `Pop`s,
//! requiring only a connection to a `CommandCenter` and access to a `DroneHub` for recharging.
//!
//! If a drone loses its connection to a command center, it becomes feral, attacking
//! nearby pops and hoarding resources.

use crate::layer1::building::Building;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_ai::{manhattan_distance, ActionType, PopAction, StartPlan};
use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;

/// State of a drone.
///
/// Indicates what the drone is currently doing, such as hauling or if it has gone feral.
///
/// # Examples
///
/// ```
/// use scale::layer1::entities::drone::{Drone, DroneState};
///
/// let my_drone = Drone {
///     state: DroneState::Idle,
/// };
///
/// assert_eq!(my_drone.state, DroneState::Idle);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DroneState {
    /// Drone is idle and waiting for tasks.
    #[default]
    Idle,
    /// Drone is currently hauling resources.
    Hauling,
    /// Drone has lost connection to the Command Center and gone feral.
    Feral,
}

/// Marker component for Drone entities.
///
/// Drones are automated workers that require power (Battery) but have no needs.
#[derive(Component, Default)]
pub struct Drone {
    /// The current operational state of the drone.
    pub state: DroneState,
}

/// Component indicating a Feral Drone.
#[derive(Component, Default)]
pub struct FeralDrone {
    /// Resources hoarded by the feral drone.
    pub hoard: Vec<crate::layer1::resources::ResourceItem>,
}

/// Component indicating a drone's connection state to a Command Center.
#[derive(Component, Default)]
pub struct GridConnection {
    /// True if the drone is currently connected to power
    pub is_connected: bool,
    /// Number of ticks the drone has been disconnected
    pub time_disconnected: u64,
}

/// Component linking a drone to a Command Center.
#[derive(Component)]
pub struct ConnectedTo(pub Entity);

/// Event fired when a drone loses connection to a Command Center.
#[derive(Event)]
pub struct DroneDisconnectedEvent {
    /// The drone entity that disconnected.
    pub drone: Entity,
}

/// Marker component for Drone Hub buildings.
///
/// Drone Hubs consume power and allow Drones to recharge.
#[derive(Component, Default)]
pub struct DroneHub;

/// Component tracking internal battery charge for Drones.
#[derive(Component, Clone, Copy, Debug)]
pub struct DroneBattery {
    /// Current charge level.
    pub current: f32,
    /// Maximum charge capacity.
    pub max: f32,
}

/// Evaluates drone actions.
/// Prioritizes Charging if battery is low (< 20%).
/// Otherwise Idles (Placeholder for Hauling).
#[allow(clippy::type_complexity)]
pub fn evaluate_drone_actions_system(
    mut commands: Commands,
    mut drone_query: Query<(Entity, &DroneBattery, &mut PopAction, &GridPosition), With<Drone>>,
    hub_query: Query<(Entity, &GridPosition, &PowerConsumer), (With<DroneHub>, With<Building>)>,
) {
    for (entity, battery, mut action, pos) in &mut drone_query {
        let battery_pct = battery.current / battery.max;

        // 1. Charge if low
        if battery_pct < 0.2 {
            if action.current != ActionType::Charge {
                // Find nearest active Hub
                let mut best_hub = None;
                let mut min_dist = i32::MAX;

                for (hub_entity, hub_pos, power) in hub_query.iter() {
                    if !power.active {
                        continue;
                    }
                    let dist = manhattan_distance(pos, hub_pos);
                    if dist < min_dist {
                        min_dist = dist;
                        best_hub = Some(hub_entity);
                    }
                }

                if let Some(hub) = best_hub {
                    action.current = ActionType::Charge;
                    action.current_utility = 1.0;
                    action.ticks_committed = 0;

                    // Trigger movement via StartPlan
                    commands.entity(entity).insert(StartPlan {
                        action: ActionType::Charge,
                        target: Some(hub),
                    });
                }
            }
            continue;
        }

        // 2. Resume Idle if fully charged (and was charging)
        if action.current == ActionType::Charge && battery_pct >= 0.99 {
            action.current = ActionType::Idle;
            action.current_utility = 0.5;
            action.ticks_committed = 0;
            commands.entity(entity).remove::<StartPlan>(); // Stop moving to charger
        }

        // 3. TODO: Haul Logic
        if action.current == ActionType::Idle {
            // Placeholder: Stay Idle
        }
    }
}

/// Checks if a drone has lost connection to its Command Center.
pub fn check_drone_connection(
    mut commands: Commands,
    mut drones: Query<(Entity, &mut Drone, &ConnectedTo, &mut GridConnection)>,
    command_centers: Query<&PowerConsumer, With<Building>>,
    mut disconnect_events: EventWriter<DroneDisconnectedEvent>,
) {
    const FERAL_THRESHOLD: u64 = 5000;

    for (entity, mut drone, connection, mut grid_connection) in drones.iter_mut() {
        let mut disconnected = false;
        // Find the command center entity
        if let Ok(power) = command_centers.get(connection.0) {
            // Check if it is powered
            if !power.active {
                disconnected = true;
            }
        } else {
            // Command center is destroyed or missing PowerConsumer
            disconnected = true;
        }

        if disconnected {
            grid_connection.is_connected = false;
            grid_connection.time_disconnected += 1;
        } else {
            grid_connection.is_connected = true;
            grid_connection.time_disconnected = 0;
        }

        if !grid_connection.is_connected && grid_connection.time_disconnected >= FERAL_THRESHOLD {
            commands.entity(entity).remove::<ConnectedTo>();
            commands.entity(entity).insert(FeralDrone::default());
            drone.state = DroneState::Feral;
            disconnect_events.send(DroneDisconnectedEvent { drone: entity });
        }
    }
}

/// Processes feral drones: hoarding nearby resources and attacking pops.
pub fn process_feral_drones(
    mut commands: Commands,
    mut feral_drones: Query<(&mut FeralDrone, &GridPosition)>,
    resources: Query<(
        Entity,
        &crate::layer1::resources::ResourceItem,
        &GridPosition,
    )>,
    mut pops: Query<
        (&mut crate::layer1::health::Health, &GridPosition),
        With<crate::layer1::pop::Pop>,
    >,
) {
    for (mut feral_drone, drone_pos) in feral_drones.iter_mut() {
        // Reproduce if we have enough hoarded resources
        if feral_drone.hoard.len() >= 5 {
            feral_drone.hoard.clear();
            commands.spawn((
                Drone {
                    state: DroneState::Feral,
                },
                FeralDrone::default(),
                *drone_pos,
                PopAction::default(),
                DroneBattery {
                    current: 100.0,
                    max: 100.0,
                },
            ));
            // Skip gathering this tick since we just reproduced
            continue;
        }

        // Hoard nearby resources (within 2 tiles distance)
        for (res_entity, resource, res_pos) in resources.iter() {
            if manhattan_distance(drone_pos, res_pos) < 2 {
                feral_drone.hoard.push(*resource);
                commands.entity(res_entity).despawn();
                break; // Only pick up one per tick
            }
        }

        // Attack nearby pops (within 2 tiles distance)
        for (mut health, pop_pos) in pops.iter_mut() {
            if manhattan_distance(drone_pos, pop_pos) < 2 {
                health.current -= 10.0;
            }
        }
    }
}

/// Charges drones when they are at a Hub and performing `ActionType::Charge`.
pub fn process_charge_system(
    mut drone_query: Query<(Entity, &mut DroneBattery, &PopAction, &GridPosition), With<Drone>>,
    hub_query: Query<(&GridPosition, &PowerConsumer), With<DroneHub>>,
) {
    for (_entity, mut battery, action, pos) in &mut drone_query {
        if action.current == ActionType::Charge {
            // Check if at any ACTIVE hub location
            let at_active_hub = hub_query
                .iter()
                .any(|(hub_pos, power)| hub_pos == pos && power.active);

            if at_active_hub {
                battery.current = (battery.current + 1.0).min(battery.max);
            }
        }
    }
}

/// Drains drone battery over time.
pub fn drone_battery_system(mut query: Query<(&mut DroneBattery, &PopAction), With<Drone>>) {
    for (mut battery, action) in &mut query {
        let drain = match action.current {
            ActionType::Idle => 0.05,
            ActionType::Charge => 0.0, // Don't drain while charging logic runs (it net gains)
            _ => 0.1,                  // Work harder
        };

        battery.current = (battery.current - drain).max(0.0);
    }
}

pub fn drone_spawner_bridge_system(
    mut commands: Commands,
    hubs: Query<
        (Entity, &GridPosition, &crate::layer1::energy::PowerConsumer),
        With<crate::layer1::drone::DroneHub>,
    >,
    drones: Query<&crate::layer1::drone::Drone>,
    _time: Res<SimulationTime>,
) {
    // Limit total drones to 3 * Hubs
    let hub_count = hubs.iter().count();
    if hub_count == 0 {
        return;
    }

    let drone_count = drones.iter().count();
    let max_drones = hub_count * 3;

    if drone_count >= max_drones {
        return;
    }

    // Spawn 1 drone per tick max
    for (hub_entity, pos, power) in hubs.iter() {
        if power.active {
            // Spawn drone
            commands.spawn((
                crate::layer1::drone::Drone {
                    state: crate::layer1::drone::DroneState::Idle,
                },
                crate::layer1::drone::ConnectedTo(hub_entity),
                crate::layer1::drone::GridConnection::default(),
                *pos,
                crate::layer1::utility_ai::PopAction::default(),
                crate::layer1::drone::DroneBattery {
                    current: 100.0,
                    max: 100.0,
                },
                crate::layer1::pop::Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                crate::layer1::utility_ai::UtilityWeights::default(),
            ));
            break; // Only one per tick
        }
    }
}

pub fn drone_work_bridge_system(
    mut query: Query<
        (
            &mut crate::layer1::utility_ai::PopAction,
            &crate::layer1::drone::DroneBattery,
        ),
        With<crate::layer1::drone::Drone>,
    >,
) {
    for (mut action, battery) in &mut query {
        // If idle and battery > 20%, start hauling
        // Drones handle charging logic in evaluate_drone_actions_system which sets action to Charge.
        // We only override Idle.
        if action.current == crate::layer1::utility_ai::ActionType::Idle && battery.current > 20.0 {
            action.current = crate::layer1::utility_ai::ActionType::Haul;
            action.current_utility = 0.8; // High utility to persist
            action.ticks_committed = 0;
        }
    }
}
