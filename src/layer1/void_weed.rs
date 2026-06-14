use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct VoidWeedStash {
    pub amount: f32,
}

#[derive(Resource, Default)]
pub struct TradeNetwork {
    pub credits: f32,
}

#[derive(Resource, Default)]
pub struct SmugglingHeat {
    pub level: f32,
}

#[derive(Event)]
pub struct MerchantArrivalEvent {
    pub merchant_type: MerchantType,
}

#[derive(PartialEq, Debug)]
pub enum MerchantType {
    Smuggler,
    Legitimate,
    Enigmatic,
}

#[derive(Event)]
pub struct PirateRaidEvent;

pub fn process_void_weed_trade_system(
    mut events: EventReader<MerchantArrivalEvent>,
    mut network: ResMut<TradeNetwork>,
    mut stashes: Query<&mut VoidWeedStash>,
    mut heat: ResMut<SmugglingHeat>,
) {
    for event in events.read() {
        if event.merchant_type == MerchantType::Smuggler {
            for mut stash in stashes.iter_mut() {
                if stash.amount > 0.0 {
                    let profit = stash.amount * 500.0;
                    network.credits += profit;
                    heat.level += stash.amount * 2.0;
                    stash.amount = 0.0;
                }
            }
        }
    }
}

pub fn evaluate_smuggling_heat_system(
    mut heat: ResMut<SmugglingHeat>,
    mut raid_events: EventWriter<PirateRaidEvent>,
) {
    if heat.level >= 100.0 {
        raid_events.send(PirateRaidEvent);
        heat.level = 0.0; // Reset heat after raid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_void_weed_smuggling_profits() {
        let mut world = World::new();
        world.insert_resource(TradeNetwork { credits: 0.0 });
        world.insert_resource(SmugglingHeat { level: 0.0 });
        world.init_resource::<Events<MerchantArrivalEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_void_weed_trade_system);

        let entity = world.spawn((VoidWeedStash { amount: 10.0 },)).id();

        world
            .resource_mut::<Events<MerchantArrivalEvent>>()
            .send(MerchantArrivalEvent {
                merchant_type: MerchantType::Smuggler,
            });

        schedule.run(&mut world);

        let network = world.resource::<TradeNetwork>();
        assert_eq!(network.credits, 5000.0); // 10.0 * 500.0 credits
        let stash = world.get::<VoidWeedStash>(entity).unwrap();
        assert_eq!(stash.amount, 0.0);
    }

    #[test]
    fn test_void_weed_smuggling_attracts_pirates() {
        let mut world = World::new();
        world.insert_resource(SmugglingHeat { level: 0.0 });
        world.init_resource::<Events<PirateRaidEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_smuggling_heat_system);

        world.insert_resource(SmugglingHeat { level: 100.0 });

        schedule.run(&mut world);

        let raid_events = world.resource::<Events<PirateRaidEvent>>();
        let mut reader = raid_events.get_cursor();
        let events: Vec<_> = reader.read(raid_events).collect();

        assert_eq!(events.len(), 1);
    }
}
