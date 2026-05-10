use bevy_ecs::prelude::*;

use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::shared::time::SimulationTime;

#[derive(Clone)]
pub struct ResourceCost {
    pub resource_type: ResourceType,
    pub amount: f32,
}

#[derive(Event)]
pub struct RansomDemandEvent {
    pub target_pop: Entity,
    pub cost: ResourceCost,
    pub deadline_tick: u64,
}

#[derive(Event)]
pub struct PayRansomEvent {
    pub target_pop: Entity,
}

#[derive(Event)]
pub struct RefuseRansomEvent {
    pub target_pop: Entity,
}

#[derive(Event)]
pub struct PopLostToPiratesEvent {
    pub pop_entity: Entity,
}

#[derive(Component)]
pub struct RansomDemand {
    pub cost: ResourceCost,
    pub deadline_tick: u64,
}

pub fn ransom_demand_system(mut commands: Commands, mut events: EventReader<RansomDemandEvent>) {
    for event in events.read() {
        commands.entity(event.target_pop).insert(RansomDemand {
            cost: event.cost.clone(),
            deadline_tick: event.deadline_tick,
        });
    }
}

pub fn process_ransom_decisions_system(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    mut pay_events: EventReader<PayRansomEvent>,
    mut refuse_events: EventReader<RefuseRansomEvent>,
    mut lost_events: EventWriter<PopLostToPiratesEvent>,
    query: Query<(Entity, &RansomDemand)>,
) {
    // Handle payments
    for event in pay_events.read() {
        if let Ok((entity, demand)) = query.get(event.target_pop) {
            let available = match demand.cost.resource_type {
                ResourceType::Food => resources.food,
                ResourceType::Wood => resources.wood,
                ResourceType::Stone => resources.stone,
                ResourceType::Metal => resources.metal,
                ResourceType::Fuel => resources.fuel,
                _ => resources.credits, // Fallback for testing/others if not explicitly mapped
            };

            if available >= demand.cost.amount {
                // Deduct resource
                match demand.cost.resource_type {
                    ResourceType::Food => resources.food -= demand.cost.amount,
                    ResourceType::Wood => resources.wood -= demand.cost.amount,
                    ResourceType::Stone => resources.stone -= demand.cost.amount,
                    ResourceType::Metal => resources.metal -= demand.cost.amount,
                    ResourceType::Fuel => resources.fuel -= demand.cost.amount,
                    _ => resources.credits -= demand.cost.amount,
                }
                commands.entity(entity).remove::<RansomDemand>();
            }
        }
    }

    // Handle refusals
    for event in refuse_events.read() {
        if query.get(event.target_pop).is_ok() {
            lost_events.send(PopLostToPiratesEvent {
                pop_entity: event.target_pop,
            });
            commands.entity(event.target_pop).despawn();
        }
    }
}

pub fn ransom_expiration_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    query: Query<(Entity, &RansomDemand)>,
    mut lost_events: EventWriter<PopLostToPiratesEvent>,
) {
    // Handle expirations
    for (entity, demand) in query.iter() {
        if time.tick > demand.deadline_tick {
            lost_events.send(PopLostToPiratesEvent { pop_entity: entity });
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::layer1::skills::{SkillType, Skills};
    use crate::shared::time::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<RansomDemandEvent>();
        app.add_event::<PayRansomEvent>();
        app.add_event::<RefuseRansomEvent>();
        app.add_event::<PopLostToPiratesEvent>();
        app.init_resource::<ColonyResources>();
        app.init_resource::<SimulationTime>();
        app.add_systems(
            Update,
            (
                ransom_demand_system,
                process_ransom_decisions_system,
                ransom_expiration_system,
            ),
        );
        app
    }

    #[test]
    fn test_ransom_demand_creation() {
        let mut app = setup_app();

        // Arrange: create a high-skilled pop
        let pop_entity = app
            .world_mut()
            .spawn((Pop, {
                let mut s = Skills::default();
                s.add_xp(SkillType::Engineering, 1000.0);
                s
            }))
            .id();

        // Act: trigger a capture event (mocked here by manually inserting the Captured component or sending an event)
        app.world_mut().send_event(RansomDemandEvent {
            target_pop: pop_entity,
            cost: ResourceCost {
                resource_type: ResourceType::Metal,
                amount: 500.0,
            }, // Rare resources
            deadline_tick: 100,
        });
        app.update();

        // Assert: the pop should now have a RansomDemand component
        assert!(app.world().entity(pop_entity).contains::<RansomDemand>());
        let demand = app.world().get::<RansomDemand>(pop_entity).unwrap();
        assert_eq!(demand.cost.amount, 500.0);
        assert_eq!(demand.cost.resource_type, ResourceType::Metal);
    }

    #[test]
    fn test_pay_ransom_returns_pop() {
        let mut app = setup_app();

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                RansomDemand {
                    cost: ResourceCost {
                        resource_type: ResourceType::Metal,
                        amount: 500.0,
                    },
                    deadline_tick: 100,
                },
            ))
            .id();

        app.world_mut().resource_mut::<ColonyResources>().metal += 1000.0;

        app.world_mut().send_event(PayRansomEvent {
            target_pop: pop_entity,
        });
        app.update();

        // Assert resources deducted
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.metal, 500.0);

        // Assert pop returned (no longer has RansomDemand)
        assert!(!app.world().entity(pop_entity).contains::<RansomDemand>());
    }

    #[test]
    fn test_refuse_ransom_loses_pop() {
        let mut app = setup_app();

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                RansomDemand {
                    cost: ResourceCost {
                        resource_type: ResourceType::Metal,
                        amount: 500.0,
                    },
                    deadline_tick: 100,
                },
            ))
            .id();

        app.world_mut().send_event(RefuseRansomEvent {
            target_pop: pop_entity,
        });
        app.update();

        // Assert pop is permanently lost (despawned)
        assert!(app.world().get_entity(pop_entity).is_err());
        let events = app
            .world()
            .get_resource::<Events<PopLostToPiratesEvent>>()
            .unwrap();
        assert_eq!(events.get_cursor().read(events).count(), 1);
    }

    #[test]
    fn test_ransom_deadline_expires() {
        let mut app = setup_app();

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                RansomDemand {
                    cost: ResourceCost {
                        resource_type: ResourceType::Metal,
                        amount: 500.0,
                    },
                    deadline_tick: 10,
                },
            ))
            .id();

        app.world_mut().resource_mut::<SimulationTime>().tick = 11;
        app.update();

        // Assert pop is permanently lost (despawned) due to missed deadline
        assert!(app.world().get_entity(pop_entity).is_err());
        let events = app
            .world()
            .get_resource::<Events<PopLostToPiratesEvent>>()
            .unwrap();
        assert_eq!(events.get_cursor().read(events).count(), 1);
    }
}
