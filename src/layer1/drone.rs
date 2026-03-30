use crate::layer1::building::Building;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_ai::{manhattan_distance, ActionType, PopAction, StartPlan};
use bevy_ecs::prelude::*;

/// Component for Drone entities.
///
/// Drones are automated workers that require power (Battery) but have no needs.
#[derive(Component, Default)]
pub struct Drone {
    /// The hub that spawned this drone.
    pub parent_hub: Option<Entity>,
    /// Whether the drone is active.
    pub is_active: bool,
}

/// Component for Drone Hub buildings.
///
/// Drone Hubs consume power and allow Drones to recharge.
#[derive(Component, Default)]
pub struct DroneHub {
    /// Maximum number of active drones this hub can support.
    pub max_bandwidth: u32,
    /// Currently active drones linked to this hub.
    pub active_drones: u32,
}

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
    mut drone_query: Query<(Entity, &DroneBattery, &mut PopAction, &GridPosition, &Drone)>,
    hub_query: Query<(Entity, &GridPosition, &PowerConsumer), (With<DroneHub>, With<Building>)>,
) {
    for (entity, battery, mut action, pos, drone_cmp) in &mut drone_query {
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

        // 3. Haul Logic handled by integration

        // If deactivated, force idle and clear utility
        if !drone_cmp.is_active && action.current != ActionType::Idle {
            action.current = ActionType::Idle;
            action.current_utility = 0.0;
            action.ticks_committed = 0;
            commands.entity(entity).remove::<StartPlan>();
        }
    }
}

/// Disables drones if their parent hub loses power or is destroyed.
pub fn drone_power_monitor_system(
    mut drone_query: Query<&mut Drone>,
    hub_query: Query<&PowerConsumer, With<DroneHub>>,
) {
    for mut drone in &mut drone_query {
        if let Some(hub_entity) = drone.parent_hub {
            if let Ok(power) = hub_query.get(hub_entity) {
                drone.is_active = power.active;
            } else {
                // Hub doesn't exist or lost power component
                drone.is_active = false;
            }
        } else {
            // No parent hub
            drone.is_active = false;
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
