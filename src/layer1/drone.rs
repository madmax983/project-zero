//! Drone Network system (Spec 116).
//!
//! Drones are automated workers that handle hauling tasks.
//! They require power (Battery) and recharge at Drone Hubs.

use crate::layer1::actions::haul::evaluate_haul;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_ai::{ActionType, PopAction, StartPlan};
use crate::layer1::utility_eval_types::UtilityAIBuffer;
use bevy_ecs::prelude::*;

/// Marker for Drone entities.
#[derive(Component, Default, Debug)]
pub struct Drone;

/// Marker for Drone Hub buildings.
#[derive(Component, Default, Debug)]
pub struct DroneHub;

/// Component tracking battery level.
#[derive(Component, Clone, Copy, Debug)]
pub struct DroneBattery {
    /// Current charge.
    pub current: f32,
    /// Maximum charge.
    pub max: f32,
}

impl Default for DroneBattery {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

/// Drains battery from active drones.
pub fn battery_drain_system(mut query: Query<(&mut DroneBattery, &PopAction), With<Drone>>) {
    for (mut battery, action) in &mut query {
        let drain = if action.current == ActionType::Charge || action.current == ActionType::Idle {
            0.01 // Low drain when idle/charging
        } else {
            0.1 // Active drain
        };
        battery.current = (battery.current - drain).max(0.0);
    }
}

/// Evaluates actions for Drones (Charge vs Haul).
pub fn evaluate_drone_actions_system(
    mut commands: Commands,
    mut drones: Query<
        (
            Entity,
            &GridPosition,
            &DroneBattery,
            &mut PopAction,
            Option<&crate::layer1::resources::Carrying>,
        ),
        With<Drone>,
    >,
    buffer: Option<Res<UtilityAIBuffer>>,
    resources: Res<ColonyResources>,
    hubs: Query<(Entity, &GridPosition), With<DroneHub>>,
) {
    let Some(buffer) = buffer else { return };
    let weights = crate::layer1::utility_types::UtilityWeights::default();

    for (entity, pos, battery, mut action, carrying) in &mut drones {
        // 1. Check Battery (Critical Priority)
        // If carrying, we might want to drop off first?
        // For simplicity: If < 10%, drop everything (despawn item?) and charge.
        // Current logic: Just switch to Charge. Carrying comp remains but `cleanup` might behave weirdly?
        // `cleanup_previous_assignment` handles assignments. `Carrying` is a component.
        // `haul_system` handles dropoff.
        // If we switch to Charge, we walk to Hub. We still hold the item.
        // When charged, we resume Haul.
        // If we die (0 battery), we drop item? (Not implemented yet).

        if battery.current < 20.0 {
            if action.current != ActionType::Charge {
                // Find nearest Hub
                let mut best_hub = None;
                let mut min_dist = i32::MAX;

                for (hub_entity, hub_pos) in &hubs {
                    let dist = crate::layer1::utility_types::manhattan_distance(pos, hub_pos);
                    if dist < min_dist {
                        min_dist = dist;
                        best_hub = Some(hub_entity);
                    }
                }

                if let Some(hub) = best_hub {
                    action.current = ActionType::Charge;
                    action.current_utility = 1.0;
                    action.ticks_committed = 0;

                    commands.entity(entity).insert(StartPlan {
                        action: ActionType::Charge,
                        target: Some(hub),
                    });
                }
            }
            continue;
        }

        // 2. If Charging, stay until full
        if action.current == ActionType::Charge && battery.current < battery.max * 0.95 {
            continue;
        }

        // 3. Haul Logic
        let haul_score = evaluate_haul(
            *pos,
            &weights,
            &buffer.items,
            &buffer.stockpiles,
            &resources,
            carrying.copied(),
        );

        if let Some((utility, target)) = haul_score {
            // Hysteresis: only switch if better
            if utility > action.current_utility + 0.1 || action.current == ActionType::Charge {
                action.current = ActionType::Haul;
                action.current_utility = utility;
                action.ticks_committed = 0;

                commands.entity(entity).insert(StartPlan {
                    action: ActionType::Haul,
                    target: Some(target),
                });
            }
        } else if action.current != ActionType::Idle && action.current != ActionType::Charge {
            // No work found, go Idle
            action.current = ActionType::Idle;
            action.current_utility = 0.0;
            commands.entity(entity).insert(StartPlan {
                action: ActionType::Idle,
                target: None,
            });
        }
    }
}

/// Recharges drones when they are at a Hub.
pub fn process_charge_system(
    mut drones: Query<(&mut DroneBattery, &GridPosition, &PopAction), With<Drone>>,
    hubs: Query<&GridPosition, With<DroneHub>>,
) {
    for (mut battery, pos, action) in &mut drones {
        if action.current == ActionType::Charge {
            // Check if at any hub
            let at_hub = hubs.iter().any(|h_pos| h_pos == pos);

            if at_hub {
                battery.current = (battery.current + 5.0).min(battery.max);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::utility_ai::ActionType;

    #[test]
    fn test_battery_default() {
        let b = DroneBattery::default();
        assert_eq!(b.current, 100.0);
        assert_eq!(b.max, 100.0);
    }

    #[test]
    fn test_battery_drain() {
        let mut world = World::new();
        let drone = world
            .spawn((
                Drone,
                DroneBattery::default(),
                PopAction {
                    current: ActionType::Haul,
                    ..Default::default()
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(battery_drain_system);
        schedule.run(&mut world);

        let b = world.get::<DroneBattery>(drone).unwrap();
        assert!(b.current < 100.0);
    }

    #[test]
    fn test_charge_system() {
        let mut world = World::new();
        // Hub at 0,0
        world.spawn((DroneHub, GridPosition { x: 0, y: 0 }));

        // Drone at 0,0, Charging
        let drone = world
            .spawn((
                Drone,
                DroneBattery {
                    current: 50.0,
                    max: 100.0,
                },
                GridPosition { x: 0, y: 0 },
                PopAction {
                    current: ActionType::Charge,
                    ..Default::default()
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_charge_system);
        schedule.run(&mut world);

        let b = world.get::<DroneBattery>(drone).unwrap();
        assert!(b.current > 50.0);
    }
}
