use crate::layer1::balance::TICKS_PER_YEAR;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::economy::resources::ResourceType;
use crate::shared::time::SimulationTime;
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

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct ParadoxEvent;

pub fn open_rift(world: &mut World, _rift: Entity, res_type: ResourceType, amount: f32) {
    if let Some(mut res) = world.get_resource_mut::<ColonyResources>() {
        res.add_resource(&res_type, amount);
    } else {
        let mut res = ColonyResources::default();
        res.add_resource(&res_type, amount);
        world.insert_resource(res);
    }

    let current_tick = world.resource::<SimulationTime>().tick;
    world.spawn(TemporalDebt {
        resource_type: res_type,
        amount,
        due_tick: current_tick + TICKS_PER_YEAR,
    });
}

pub fn pay_temporal_debt(world: &mut World) {
    let mut paid_debts = Vec::new();

    let mut debts_to_check = Vec::new();
    for (debt_entity, debt) in world.query::<(Entity, &TemporalDebt)>().iter(world) {
        debts_to_check.push((debt_entity, debt.resource_type, debt.amount));
    }

    if let Some(mut res) = world.get_resource_mut::<ColonyResources>() {
        for (debt_entity, r_type, amount) in debts_to_check {
            if res.get_amount(r_type) >= amount {
                res.consume(r_type, amount);
                paid_debts.push(debt_entity);
            }
        }
    }

    for debt_entity in paid_debts {
        world.despawn(debt_entity);
    }
}

pub fn check_temporal_debts(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;
    let mut defaulted_debts = Vec::new();

    for (debt_entity, debt) in world.query::<(Entity, &TemporalDebt)>().iter(world) {
        if current_tick > debt.due_tick {
            defaulted_debts.push(debt_entity);
        }
    }

    for debt_entity in defaulted_debts {
        world.despawn(debt_entity);
        if world.contains_resource::<Events<ParadoxEvent>>() {
            world.send_event(ParadoxEvent);
        }
    }
}

pub fn check_temporal_debts_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    debts: Query<(Entity, &TemporalDebt)>,
    mut paradox_events: EventWriter<ParadoxEvent>,
) {
    for (entity, debt) in debts.iter() {
        if time.tick > debt.due_tick {
            paradox_events.send(ParadoxEvent);
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rift_grants_resources_and_incurs_debt() {
        let mut world = World::new();
        let sim_time = SimulationTime {
            tick: 0,
            speed: crate::shared::time::SimSpeed::Normal,
        };
        world.insert_resource(sim_time);
        world.insert_resource(ColonyResources::default());

        let rift_entity = world.spawn(TemporalRift { active: true }).id();

        open_rift(&mut world, rift_entity, ResourceType::Wood, 50.0);

        let res = world.get_resource::<ColonyResources>().unwrap();
        assert_eq!(res.get_amount(ResourceType::Wood), 50.0);

        let mut debt_query = world.query::<&TemporalDebt>();
        let debt = debt_query.iter(&world).next().unwrap();
        assert_eq!(debt.resource_type, ResourceType::Wood);
        assert_eq!(debt.amount, 50.0);
        assert_eq!(debt.due_tick, TICKS_PER_YEAR);
    }

    #[test]
    fn test_rift_closes_loop_on_payment() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.wood = 50.0;
        world.insert_resource(res);

        world.spawn(TemporalDebt {
            resource_type: ResourceType::Wood,
            amount: 50.0,
            due_tick: 100,
        });

        pay_temporal_debt(&mut world);

        let res = world.get_resource::<ColonyResources>().unwrap();
        assert_eq!(res.get_amount(ResourceType::Wood), 0.0);

        let mut debt_query = world.query::<&TemporalDebt>();
        assert_eq!(debt_query.iter(&world).count(), 0);
    }

    #[test]
    fn test_unpaid_debt_causes_paradox() {
        let mut world = World::new();
        let sim_time = SimulationTime {
            tick: 101,
            speed: crate::shared::time::SimSpeed::Normal,
        };
        world.insert_resource(sim_time);
        world.init_resource::<Events<ParadoxEvent>>();

        world.spawn(TemporalDebt {
            resource_type: ResourceType::Wood,
            amount: 50.0,
            due_tick: 100,
        });

        check_temporal_debts(&mut world);

        let events = world.resource::<Events<ParadoxEvent>>();
        #[allow(deprecated)]
        let mut reader = events.get_reader();
        let emitted: Vec<_> = reader.read(events).collect();
        assert_eq!(emitted.len(), 1);

        // debt entity despawned
        let mut debt_query = world.query::<&TemporalDebt>();
        assert_eq!(debt_query.iter(&world).count(), 0);
    }
}
