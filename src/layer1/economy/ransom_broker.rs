use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::ColonyResources;
use crate::shared::time::SimulationTime;

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

#[derive(Component)]
pub struct RansomDemand {
    pub cost: u32,
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
    // Handle payments
    for event in pay_events.read() {
        if let Ok((entity, demand)) = query.get(event.target_pop) {
            // Using metal as the rare resource since RareMetals isn't available
            if resources.metal >= demand.cost as f32 {
                resources.metal -= demand.cost as f32;
                commands.entity(entity).remove::<RansomDemand>();
            }
        }
    }

    // Handle refusals
    for event in refuse_events.read() {
        if query.get(event.target_pop).is_ok() {
            commands.entity(event.target_pop).despawn();
        }
    }

    // Handle expirations
    for (entity, demand) in query.iter() {
        if time.tick > demand.deadline_tick {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::skills::Skills;
    use crate::layer1::economy::resources::ColonyResources;
    use crate::shared::time::{SimulationTime, SimSpeed};

    fn setup_world() -> (World, Schedule) {
        let mut world = World::new();
        world.init_resource::<Events<RansomDemandEvent>>();
        world.init_resource::<Events<PayRansomEvent>>();
        world.init_resource::<Events<RefuseRansomEvent>>();
        world.init_resource::<ColonyResources>();
        world.insert_resource(SimulationTime {
            tick: 0,
            speed: SimSpeed::Paused,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems((ransom_demand_system, process_ransom_decisions_system));

        (world, schedule)
    }

    #[test]
    fn test_ransom_demand_creation() {
        let (mut world, mut schedule) = setup_world();

        let pop_entity = world.spawn((
            Pop,
            Skills::default(),
        )).id();

        world.send_event(RansomDemandEvent {
            target_pop: pop_entity,
            cost: 500,
            deadline_tick: 100,
        });
        schedule.run(&mut world);

        assert!(world.entity(pop_entity).contains::<RansomDemand>());
        let demand = world.get::<RansomDemand>(pop_entity).unwrap();
        assert_eq!(demand.cost, 500);
    }

    #[test]
    fn test_pay_ransom_returns_pop() {
        let (mut world, mut schedule) = setup_world();

        let pop_entity = world.spawn((
            Pop,
            RansomDemand {
                cost: 500,
                deadline_tick: 100,
            },
        )).id();

        world.resource_mut::<ColonyResources>().metal = 1000.0;

        world.send_event(PayRansomEvent { target_pop: pop_entity });
        schedule.run(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.metal, 500.0);

        assert!(!world.entity(pop_entity).contains::<RansomDemand>());
    }

    #[test]
    fn test_refuse_ransom_loses_pop() {
        let (mut world, mut schedule) = setup_world();

        let pop_entity = world.spawn((
            Pop,
            RansomDemand {
                cost: 500,
                deadline_tick: 100,
            },
        )).id();

        world.send_event(RefuseRansomEvent { target_pop: pop_entity });
        schedule.run(&mut world);

        assert!(world.get_entity(pop_entity).is_err());
    }

    #[test]
    fn test_ransom_deadline_expires() {
        let (mut world, mut schedule) = setup_world();

        let pop_entity = world.spawn((
            Pop,
            RansomDemand {
                cost: 500,
                deadline_tick: 10,
            },
        )).id();

        world.resource_mut::<SimulationTime>().tick = 11;
        schedule.run(&mut world);

        assert!(world.get_entity(pop_entity).is_err());
    }
}
