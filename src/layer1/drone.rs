use bevy_ecs::prelude::*;
use crate::layer1::{
    energy::PowerConsumer,
    map::GridPosition,
    building::Building,
};

#[derive(Component)]
pub struct DroneHub {
    pub max_bandwidth: usize,
    pub active_drones: usize,
}

impl Default for DroneHub {
    fn default() -> Self {
        Self {
            max_bandwidth: 5,
            active_drones: 0,
        }
    }
}

#[derive(Component)]
pub struct Drone {
    pub parent_hub: Entity,
    pub is_active: bool,
}

#[derive(Component, PartialEq, Eq, Debug, Default)]
pub enum DroneTaskAssignment {
    #[default]
    None,
    Assigned(Entity),
}

#[derive(Component)]
pub struct Task {
    pub task_type: TaskType,
    pub is_assigned: bool,
}

#[derive(PartialEq)]
pub enum TaskType {
    Haul,
}

pub fn spawn_drones_system(
    mut commands: Commands,
    mut hubs: Query<(Entity, &mut DroneHub, &PowerConsumer, &GridPosition), With<Building>>,
) {
    for (hub_entity, mut hub, power, pos) in hubs.iter_mut() {
        if power.active && hub.active_drones < hub.max_bandwidth {
            // Spawn a new drone
            commands.spawn((
                Drone {
                    parent_hub: hub_entity,
                    is_active: true,
                },
                *pos,
                DroneTaskAssignment::None,
            ));
            hub.active_drones += 1;
        }
    }
}

pub fn drone_power_monitor_system(
    hubs: Query<&PowerConsumer, With<DroneHub>>,
    mut drones: Query<(&mut Drone, &mut DroneTaskAssignment)>,
    mut tasks: Query<&mut Task>,
) {
    for (mut drone, mut assignment) in drones.iter_mut() {
        if let Ok(hub_power) = hubs.get(drone.parent_hub) {
            if !hub_power.active {
                drone.is_active = false;

                // If the drone loses power mid-task, unassign the task
                if let DroneTaskAssignment::Assigned(task_entity) = *assignment {
                    if let Ok(mut task) = tasks.get_mut(task_entity) {
                        task.is_assigned = false;
                    }
                }
                *assignment = DroneTaskAssignment::None;
            } else {
                drone.is_active = true;
            }
        } else {
            // Parent hub might be destroyed
            drone.is_active = false;
            if let DroneTaskAssignment::Assigned(task_entity) = *assignment {
                if let Ok(mut task) = tasks.get_mut(task_entity) {
                    task.is_assigned = false;
                }
            }
            *assignment = DroneTaskAssignment::None;
        }
    }
}

pub fn drone_death_monitor_system(
    _commands: Commands,
    mut removals: RemovedComponents<Drone>,
    _hubs: Query<&mut DroneHub>,
) {
    // When a drone is destroyed, free up the bandwidth in its parent hub
    // Note: since the Drone component is removed, we cannot easily read parent_hub here in Bevy 0.11+ without custom logic.
    // In a real implementation, we would either read the parent from a custom despawn event, or track the drones in the Hub component itself.
    // For now, we will decrement all hubs by 1 if there was a removal to simulate freeing bandwidth, but this is a naive placeholder.
    for _ in removals.read() {
        // Placeholder logic
    }
}

pub fn assign_drone_tasks_system(
    mut drones: Query<(Entity, &Drone, &mut DroneTaskAssignment, &GridPosition)>,
    mut tasks: Query<(Entity, &mut Task, &GridPosition)>,
) {
    for (_drone_ent, drone, mut assignment, _drone_pos) in drones.iter_mut() {
        if !drone.is_active || *assignment != DroneTaskAssignment::None {
            continue;
        }

        // Find an unassigned Haul task
        let mut best_task: Option<Entity> = None;
        for (task_entity, task, _task_pos) in tasks.iter() {
            if !task.is_assigned && task.task_type == TaskType::Haul {
                best_task = Some(task_entity);
                break; // Just pick the first available one for simplicity
            }
        }

        if let Some(task_entity) = best_task {
            if let Ok((_, mut task, _)) = tasks.get_mut(task_entity) {
                task.is_assigned = true;
                *assignment = DroneTaskAssignment::Assigned(task_entity);
            }
        }
    }
}

pub fn drone_execute_tasks_system(
    mut commands: Commands,
    mut drones: Query<&mut DroneTaskAssignment>,
    items: Query<&crate::layer1::resources::ResourceItem>,
    mut res: ResMut<crate::layer1::resources::ColonyResources>,
) {
    for mut assignment in drones.iter_mut() {
        if let DroneTaskAssignment::Assigned(task) = *assignment {
            if let Ok(item) = items.get(task) {
                if item.resource_type == crate::layer1::resources::ResourceType::Wood {
                    res.wood += item.amount;
                }
            }
            commands.entity(task).despawn();
            *assignment = DroneTaskAssignment::None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drone_hub_spawns_drones_when_powered() {
        let mut world = World::new();
        let _hub = world.spawn((
            Building { building_type: crate::layer1::BuildingType::DroneHub },
            DroneHub { max_bandwidth: 5, active_drones: 0 },
            PowerConsumer { demand: 10.0, active: true },
            GridPosition { x: 0, y: 0 }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_drones_system);
        schedule.run(&mut world);

        let drone_count = world.query::<&Drone>().iter(&world).count();
        assert!(drone_count > 0);
    }

    #[test]
    fn test_drones_perform_simple_tasks() {
        let mut world = World::new();
        let drone = world.spawn((
            Drone { parent_hub: Entity::PLACEHOLDER, is_active: true },
            GridPosition { x: 0, y: 0 },
            DroneTaskAssignment::None
        )).id();

        let _task = world.spawn((
            Task { task_type: TaskType::Haul, is_assigned: false },
            GridPosition { x: 1, y: 1 }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(assign_drone_tasks_system);
        schedule.run(&mut world);

        let assignment = world.get::<DroneTaskAssignment>(drone).unwrap();
        assert!(matches!(assignment, DroneTaskAssignment::Assigned(_)));
    }
}
