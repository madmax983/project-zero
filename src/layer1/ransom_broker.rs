use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::shared::time::SimulationTime;
use bevy::prelude::*;

#[derive(Event)]
pub struct RansomDemandEvent {
    pub target_pop: Entity,
    pub cost: u32,
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
pub struct PopRansomedEvent {
    pub target_pop: Entity,
}

#[derive(Event)]
pub struct PopLostToPiratesEvent {
    pub target_pop: Entity,
}

#[derive(Component)]
pub struct RansomDemand {
    pub cost: u32,
    pub deadline_tick: u64,
}

pub fn ransom_demand_system(mut commands: Commands, mut events: EventReader<RansomDemandEvent>) {
    for event in events.read() {
        commands.entity(event.target_pop).insert(RansomDemand {
            cost: event.cost,
            deadline_tick: event.deadline_tick,
        });
    }
}

pub fn process_ransom_decisions_system(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    time: Res<SimulationTime>,
    mut pay_events: EventReader<PayRansomEvent>,
    mut refuse_events: EventReader<RefuseRansomEvent>,
    mut ransomed_events: EventWriter<PopRansomedEvent>,
    mut lost_events: EventWriter<PopLostToPiratesEvent>,
    query: Query<(Entity, &RansomDemand)>,
) {
    let mut resolved_entities = bevy::utils::HashSet::new();

    // Handle payments
    for event in pay_events.read() {
        if let Ok((entity, demand)) = query.get(event.target_pop) {
            if resources.get_amount(ResourceType::HyperValuable) >= demand.cost as f32 {
                resources.consume(ResourceType::HyperValuable, demand.cost as f32);
                commands.entity(entity).remove::<RansomDemand>();
                resolved_entities.insert(entity);
                ransomed_events.send(PopRansomedEvent { target_pop: entity });
            }
        }
    }

    // Handle refusals
    for event in refuse_events.read() {
        if query.get(event.target_pop).is_ok() && !resolved_entities.contains(&event.target_pop) {
            commands.entity(event.target_pop).despawn();
            resolved_entities.insert(event.target_pop);
            lost_events.send(PopLostToPiratesEvent { target_pop: event.target_pop });
        }
    }

    // Handle expirations
    for (entity, demand) in query.iter() {
        if time.tick > demand.deadline_tick && !resolved_entities.contains(&entity) {
            commands.entity(entity).despawn();
            resolved_entities.insert(entity);
            lost_events.send(PopLostToPiratesEvent { target_pop: entity });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::{ColonyResources, ResourceType};
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::shared::time::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<RansomDemandEvent>();
        app.add_event::<PayRansomEvent>();
        app.add_event::<RefuseRansomEvent>();
        app.add_event::<PopRansomedEvent>();
        app.add_event::<PopLostToPiratesEvent>();
        app.init_resource::<ColonyResources>();
        app.init_resource::<SimulationTime>();
        app.add_systems(
            Update,
            (ransom_demand_system, process_ransom_decisions_system),
        );
        app
    }

    #[test]
    fn test_ransom_demand_creation() {
        let mut app = setup_app();

        // Arrange: create a high-skilled pop
        let pop_entity = app.world_mut().spawn((Pop, Skills::default())).id();
        app.world_mut()
            .get_mut::<Skills>(pop_entity)
            .unwrap()
            .add_xp(SkillType::Engineering, 10.0);

        // Act: trigger a capture event
        app.world_mut().send_event(RansomDemandEvent {
            target_pop: pop_entity,
            cost: 500,
            deadline_tick: 100,
        });
        app.update();

        // Assert: the pop should now have a RansomDemand component
        assert!(app.world().entity(pop_entity).contains::<RansomDemand>());
        let demand = app.world().get::<RansomDemand>(pop_entity).unwrap();
        assert_eq!(demand.cost, 500);
    }

    #[test]
    fn test_pay_ransom_returns_pop() {
        let mut app = setup_app();

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                RansomDemand {
                    cost: 500,
                    deadline_tick: 100,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<ColonyResources>()
            .max_hyper_valuable = 1000.0;
        app.world_mut()
            .resource_mut::<ColonyResources>()
            .add_hyper_valuable(1000.0);

        app.world_mut().send_event(PayRansomEvent {
            target_pop: pop_entity,
        });
        app.update();

        // Assert resources deducted
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.get_amount(ResourceType::HyperValuable), 500.0);

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
                    cost: 500,
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
    }

    #[test]
    fn test_ransom_deadline_expires() {
        let mut app = setup_app();

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                RansomDemand {
                    cost: 500,
                    deadline_tick: 10,
                },
            ))
            .id();

        app.world_mut().resource_mut::<SimulationTime>().tick = 11;
        app.update();

        // Assert pop is permanently lost (despawned) due to missed deadline
        assert!(app.world().get_entity(pop_entity).is_err());
    }
}
