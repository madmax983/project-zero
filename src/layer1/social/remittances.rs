use bevy_ecs::prelude::*;
use crate::layer1::factions::FactionId;
use crate::layer1::economy::Wallet;

#[derive(Event, Debug, Clone)]
pub struct MigrantArrivalEvent {
    pub faction: FactionId,
}

#[derive(Component, Debug, Clone)]
pub struct MigrantFamilyInfo {
    pub home_faction: FactionId,
    pub remittance_target: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Homesick;

#[derive(Resource, Default)]
pub struct FactionRemittances(pub std::collections::HashMap<FactionId, f32>);

pub fn process_remittances_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &MigrantFamilyInfo, &mut Wallet, Option<&Homesick>)>,
    mut total_remittances: ResMut<FactionRemittances>,
    mut arrival_events: EventWriter<MigrantArrivalEvent>,
) {
    for (entity, info, mut wallet, homesick) in pops.iter_mut() {
        if wallet.credits >= info.remittance_target {
            wallet.credits -= info.remittance_target;

            let current = total_remittances.0.entry(info.home_faction).or_insert(0.0);
            *current += info.remittance_target;

            if homesick.is_some() {
                commands.entity(entity).remove::<Homesick>();
            }
        } else if homesick.is_none() {
            commands.entity(entity).insert(Homesick);
        }
    }

    let threshold = 100.0;
    for (faction, amount) in total_remittances.0.iter_mut() {
        if *amount >= threshold {
            *amount -= threshold;
            arrival_events.send(MigrantArrivalEvent { faction: *faction });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::factions::FactionId;
    use crate::layer1::economy::Wallet;

    #[test]
    fn test_pop_sends_remittance_and_maintains_morale() {
        // Arrange: Migrant pop with sufficient personal wealth/resources
        let mut world = World::new();
        world.insert_resource(FactionRemittances::default());
        world.insert_resource(Events::<MigrantArrivalEvent>::default());

        let pop = world.spawn((
            MigrantFamilyInfo {
                home_faction: FactionId::FarmersGuild,
                remittance_target: 50.0,
            },
            Wallet { credits: 100.0 },
        )).id();

        // Act: Process remittance cycle
        let mut schedule = Schedule::default();
        schedule.add_systems(process_remittances_system);
        schedule.run(&mut world);

        // Assert: Pop loses a percentage of wealth, 'Homesick' need remains satisfied
        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 50.0);
        assert!(world.get::<Homesick>(pop).is_none());

        let remittances = world.resource::<FactionRemittances>();
        assert_eq!(*remittances.0.get(&FactionId::FarmersGuild).unwrap_or(&0.0), 50.0);
    }

    #[test]
    fn test_failed_remittance_causes_homesick_depression() {
        // Arrange: Migrant pop with 0 wealth
        let mut world = World::new();
        world.insert_resource(FactionRemittances::default());
        world.insert_resource(Events::<MigrantArrivalEvent>::default());

        let pop = world.spawn((
            MigrantFamilyInfo {
                home_faction: FactionId::FarmersGuild,
                remittance_target: 50.0,
            },
            Wallet { credits: 10.0 }, // Not enough!
        )).id();

        // Act: Process remittance cycle
        let mut schedule = Schedule::default();
        schedule.add_systems(process_remittances_system);
        schedule.run(&mut world);

        // Assert: Pop fails to send remittance, gains 'Homesick' depression mood modifier
        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 10.0); // Credits unchanged
        assert!(world.get::<Homesick>(pop).is_some()); // Gains Homesick
    }

    #[test]
    fn test_high_remittances_trigger_migrant_arrival() {
        // Arrange: High total volume of remittances sent to a specific faction
        let mut world = World::new();
        world.insert_resource(FactionRemittances::default());
        world.insert_resource(Events::<MigrantArrivalEvent>::default());

        // Spawn multiple pops to trigger the threshold
        for _ in 0..3 {
            world.spawn((
                MigrantFamilyInfo {
                    home_faction: FactionId::MinersGuild,
                    remittance_target: 50.0,
                },
                Wallet { credits: 100.0 },
            ));
        }

        // Act: Process diplomatic/migration triggers
        let mut schedule = Schedule::default();
        schedule.add_systems(process_remittances_system);
        schedule.run(&mut world);

        // Assert: A new `MigrantArrivalEvent` is fired from the destination faction
        let events = world.resource::<Events<MigrantArrivalEvent>>();
        let mut reader = events.get_cursor();
        let events_list: Vec<_> = reader.read(events).collect();

        assert_eq!(events_list.len(), 1);
        assert_eq!(events_list[0].faction, FactionId::MinersGuild);

        // Residual should be 150 - 100 = 50
        let remittances = world.resource::<FactionRemittances>();
        assert_eq!(*remittances.0.get(&FactionId::MinersGuild).unwrap_or(&0.0), 50.0);
    }
}
