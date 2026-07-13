use crate::layer1::economy::resources::ResourceType;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct TemporalRift {
    pub active: bool,
}

#[derive(Component)]
pub struct TemporalDebt {
    pub resource_type: ResourceType,
    pub amount: f32,
    pub due_tick: u64,
}

#[derive(Event)]
pub struct ParadoxEvent;

#[derive(Event)]
pub struct OpenRiftEvent {
    pub rift: Entity,
    pub resource_type: ResourceType,
    pub amount: f32,
}

#[derive(Event)]
pub struct PayTemporalDebtEvent {
    pub debt_entity: Entity,
}

pub fn open_rift_system(
    mut commands: Commands,
    mut events: EventReader<OpenRiftEvent>,
    mut resources: ResMut<crate::layer1::economy::resources::ColonyResources>,
    sim_time: Res<crate::shared::time::SimulationTime>,
) {
    for event in events.read() {
        resources.add_resource(&event.resource_type, event.amount);

        commands.spawn(TemporalDebt {
            resource_type: event.resource_type,
            amount: event.amount,
            due_tick: sim_time.tick + crate::layer1::balance::TICKS_PER_YEAR,
        });
    }
}

pub fn pay_temporal_debt_system(
    mut commands: Commands,
    mut events: EventReader<PayTemporalDebtEvent>,
    mut resources: ResMut<crate::layer1::economy::resources::ColonyResources>,
    debts: Query<&TemporalDebt>,
) {
    for event in events.read() {
        if let Ok(debt) = debts.get(event.debt_entity) {
            if resources.get_amount(debt.resource_type) >= debt.amount {
                resources.consume(debt.resource_type, debt.amount);
                commands.entity(event.debt_entity).despawn();
            }
        }
    }
}

pub fn check_temporal_debts_system(
    mut commands: Commands,
    sim_time: Res<crate::shared::time::SimulationTime>,
    debts: Query<(Entity, &TemporalDebt)>,
    mut events: EventWriter<ParadoxEvent>,
) {
    for (entity, debt) in debts.iter() {
        if sim_time.tick > debt.due_tick {
            events.send(ParadoxEvent);
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::balance::TICKS_PER_YEAR;
    use crate::layer1::economy::resources::{ColonyResources, ResourceType};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_rift_grants_resources_and_incurs_debt() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });
        world.insert_resource(Events::<OpenRiftEvent>::default());
        world.insert_resource(Events::<ParadoxEvent>::default());

        let rift_entity = world.spawn(TemporalRift { active: true }).id();

        world
            .resource_mut::<Events<OpenRiftEvent>>()
            .send(OpenRiftEvent {
                rift: rift_entity,
                resource_type: ResourceType::Food,
                amount: 50.0,
            });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(open_rift_system);
        schedule.run(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.food, 50.0);

        let mut debt_query = world.query::<&TemporalDebt>();
        let debt = debt_query.iter(&world).next().unwrap();
        assert_eq!(debt.resource_type, ResourceType::Food);
        assert_eq!(debt.amount, 50.0);
        assert_eq!(debt.due_tick, TICKS_PER_YEAR);
    }

    #[test]
    fn test_rift_closes_loop_on_payment() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.food += 50.0;
        world.insert_resource(resources);

        let debt_entity = world
            .spawn(TemporalDebt {
                resource_type: ResourceType::Food,
                amount: 50.0,
                due_tick: 100,
            })
            .id();

        world.insert_resource(Events::<PayTemporalDebtEvent>::default());
        world
            .resource_mut::<Events<PayTemporalDebtEvent>>()
            .send(PayTemporalDebtEvent { debt_entity });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(pay_temporal_debt_system);
        schedule.run(&mut world);
        // (will be added in implementation)

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.food, 10.0);

        let mut debt_query = world.query::<&TemporalDebt>();
        assert_eq!(debt_query.iter(&world).count(), 0);
    }

    #[test]
    fn test_unpaid_debt_causes_paradox() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 101,
            ..Default::default()
        });
        world.insert_resource(Events::<ParadoxEvent>::default());

        world.spawn(TemporalDebt {
            resource_type: ResourceType::Food,
            amount: 50.0,
            due_tick: 100,
        });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(check_temporal_debts_system);
        schedule.run(&mut world);
        // (will be added in implementation)

        let paradox_events = world.resource::<Events<ParadoxEvent>>();
        let mut reader = paradox_events.get_cursor();
        let events: Vec<_> = reader.read(paradox_events).collect();
        assert_eq!(events.len(), 1);

        let mut debt_query = world.query::<&TemporalDebt>();
        assert_eq!(debt_query.iter(&world).count(), 0);
    }
}
