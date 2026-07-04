use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ResourceItem, ResourceType};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Storage {
    pub capacity: u32,
    pub current_amount: u32,
    pub resource_type: ResourceType,
}

#[derive(Event)]
pub struct StoreResourceEvent {
    pub target: Entity,
    pub resource_type: ResourceType,
    pub amount: u32,
}

pub fn process_storage_insertion_system(
    mut commands: Commands,
    mut events: EventReader<StoreResourceEvent>,
    mut storage_query: Query<(&mut Storage, &GridPosition)>,
) {
    for event in events.read() {
        if let Ok((mut storage, pos)) = storage_query.get_mut(event.target) {
            if storage.resource_type == event.resource_type {
                let available_space = storage.capacity.saturating_sub(storage.current_amount);

                if available_space >= event.amount {
                    storage.current_amount += event.amount;
                } else {
                    // Fill up what we can
                    storage.current_amount = storage.capacity;

                    // Drop the rest
                    let excess = event.amount - available_space;
                    commands.spawn((
                        ResourceItem {
                            resource_type: event.resource_type,
                            amount: excess as f32,
                        },
                        GridPosition { x: pos.x, y: pos.y },
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use bevy_ecs::schedule::Schedule;

    #[test]
    fn test_storage_cannot_exceed_capacity() {
        let mut world = World::new();
        world.init_resource::<Events<StoreResourceEvent>>();

        // Let's create a schedule to run the system since tests use App in the spec but we don't have App easily.
        let mut schedule = Schedule::default();
        schedule.add_systems(process_storage_insertion_system);

        let storage_ent = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Storage {
                    capacity: 100,
                    current_amount: 90,
                    resource_type: ResourceType::Stone,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world
            .resource_mut::<Events<StoreResourceEvent>>()
            .send(StoreResourceEvent {
                target: storage_ent,
                resource_type: ResourceType::Stone,
                amount: 20,
            });

        schedule.run(&mut world);

        let storage = world.get::<Storage>(storage_ent).unwrap();
        assert_eq!(storage.current_amount, 100);
    }

    #[test]
    fn test_excess_resources_spawn_as_ground_nodes() {
        let mut world = World::new();
        world.init_resource::<Events<StoreResourceEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_storage_insertion_system);

        let storage_ent = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Storage {
                    capacity: 100,
                    current_amount: 90,
                    resource_type: ResourceType::Stone,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world
            .resource_mut::<Events<StoreResourceEvent>>()
            .send(StoreResourceEvent {
                target: storage_ent,
                resource_type: ResourceType::Stone,
                amount: 20,
            });

        schedule.run(&mut world);

        // The remaining 10 stone should be spawned on the floor
        let mut query = world.query::<(&ResourceItem, &GridPosition)>();
        let mut found = false;
        for (node, pos) in query.iter(&world) {
            if pos.x == 5
                && pos.y == 5
                && node.resource_type == ResourceType::Stone
                && (node.amount - 10.0).abs() < f32::EPSILON
            {
                found = true;
                break;
            }
        }
        assert!(found, "Excess resource was not dropped on the ground.");
    }
}
