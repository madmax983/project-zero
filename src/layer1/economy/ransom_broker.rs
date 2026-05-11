use bevy_ecs::prelude::*;
use bevy::prelude::DespawnRecursiveExt;
use crate::layer1::economy::resources::ColonyResources;
use crate::shared::time::SimulationTime;

#[derive(Event)]
pub struct RansomDemandEvent {
    pub target_pop: Entity,
    pub cost: f32,
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

#[derive(Component)]
pub struct RansomDemand {
    pub cost: f32,
    pub deadline_tick: u64,
}

pub fn ransom_demand_system(
    mut commands: Commands,
    mut events: EventReader<RansomDemandEvent>,
) {
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
    query: Query<(Entity, &RansomDemand)>,
) {
    let mut paid_entities = std::collections::HashSet::new();

    // Handle payments
    for event in pay_events.read() {
        if let Ok((entity, demand)) = query.get(event.target_pop) {
            // Note: Using rare metals (or scrap/metal depending on available types in ColonyResources)
            // Using metal for the moment
            if resources.metal >= demand.cost {
                resources.metal -= demand.cost;
                commands.entity(entity).remove::<RansomDemand>();
                paid_entities.insert(entity);
            }
        }
    }

    // Handle refusals
    for event in refuse_events.read() {
        if query.get(event.target_pop).is_ok() && !paid_entities.contains(&event.target_pop) {
            commands.entity(event.target_pop).despawn_recursive();
        }
    }

    // Handle expirations
    for (entity, demand) in query.iter() {
        if time.tick > demand.deadline_tick && !paid_entities.contains(&entity) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::skills::{Skills, SkillType};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<RansomDemandEvent>();
        app.add_event::<PayRansomEvent>();
        app.add_event::<RefuseRansomEvent>();
        app.init_resource::<ColonyResources>();
        app.init_resource::<SimulationTime>();
        app.add_systems(Update, (ransom_demand_system, process_ransom_decisions_system));
        app
    }

    #[test]
    fn test_ransom_demand_creation() {
        let mut app = setup_app();

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 10.0);

        // Arrange: create a high-skilled pop
        let pop_entity = app.world_mut().spawn((
            Pop,
            skills,
        )).id();

        // Act: trigger a capture event
        app.world_mut().send_event(RansomDemandEvent {
            target_pop: pop_entity,
            cost: 500.0, // Rare resources
            deadline_tick: 100,
        });
        app.update();

        // Assert: the pop should now have a RansomDemand component
        assert!(app.world().entity(pop_entity).contains::<RansomDemand>());
        let demand = app.world().get::<RansomDemand>(pop_entity).unwrap();
        assert_eq!(demand.cost, 500.0);
    }

    #[test]
    fn test_pay_ransom_returns_pop() {
        let mut app = setup_app();

        let pop_entity = app.world_mut().spawn((
            Pop,
            RansomDemand {
                cost: 500.0,
                deadline_tick: 100,
            },
        )).id();

        app.world_mut().resource_mut::<ColonyResources>().metal = 1000.0;

        app.world_mut().send_event(PayRansomEvent { target_pop: pop_entity });
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

        let pop_entity = app.world_mut().spawn((
            Pop,
            RansomDemand {
                cost: 500.0,
                deadline_tick: 100,
            },
        )).id();

        app.world_mut().send_event(RefuseRansomEvent { target_pop: pop_entity });
        app.update();

        // Assert pop is permanently lost (despawned)
        assert!(app.world().get_entity(pop_entity).is_err());
    }

    #[test]
    fn test_ransom_deadline_expires() {
        let mut app = setup_app();

        let pop_entity = app.world_mut().spawn((
            Pop,
            RansomDemand {
                cost: 500.0,
                deadline_tick: 10,
            },
        )).id();

        app.world_mut().resource_mut::<SimulationTime>().tick = 11;
        app.update();

        // Assert pop is permanently lost (despawned) due to missed deadline
        assert!(app.world().get_entity(pop_entity).is_err());
    }
}
