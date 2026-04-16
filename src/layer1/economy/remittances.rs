use crate::layer1::economy::Wallet;
use crate::layer1::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
/// Contains information about a pop's family back home and their remittance goals.
pub struct MigrantFamilyInfo {
    /// Faction ID of their home system.
    pub home_faction: Entity,
    /// Target amount to send back home.
    pub remittance_target: f32,
}

#[derive(Event, Debug, Clone)]
/// Event triggered when a new migrant arrives from a specific faction.
pub struct MigrantArrivalEvent {
    /// Faction ID of their home system.
    pub home_faction: Entity,
}

#[derive(Resource, Default)]
/// Global tracker for total remittances sent to different factions.
pub struct RemittanceTracker {
    /// Sent credits mapped by faction entity ID.
    pub sent: bevy_utils::HashMap<Entity, f32>,
}

/// Processes scheduled offworld remittances, adding them to local savings.
pub fn process_remittances_system(
    mut tracker: ResMut<RemittanceTracker>,
    mut pops: Query<(&MigrantFamilyInfo, &mut Wallet, &mut Morale)>,
    mut events: EventWriter<MigrantArrivalEvent>,
) {
    for (info, mut wallet, mut morale) in pops.iter_mut() {
        if wallet.credits >= info.remittance_target {
            wallet.credits -= info.remittance_target;
            *tracker.sent.entry(info.home_faction).or_insert(0.0) += info.remittance_target;
        } else {
            morale.add_modifier(MoodModifier {
                label: "Homesick".to_string(),
                value: -0.2,
                duration: 100,
            });
        }
    }

    // Process triggers and subtract threshold
    for (faction, amount) in tracker.sent.iter_mut() {
        while *amount >= 100.0 {
            events.send(MigrantArrivalEvent {
                home_faction: *faction,
            });
            *amount -= 100.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_pop_sends_remittance_and_maintains_morale() {
        let mut world = World::new();
        world.init_resource::<RemittanceTracker>();
        let faction = world.spawn_empty().id();

        let pop = world
            .spawn((
                MigrantFamilyInfo {
                    home_faction: faction,
                    remittance_target: 10.0,
                },
                Wallet { credits: 20.0 },
                Morale::default(),
            ))
            .id();

        world.init_resource::<Events<MigrantArrivalEvent>>();
        world.run_system_once(process_remittances_system).unwrap();

        let wallet = world.get::<Wallet>(pop).unwrap();
        let morale = world.get::<Morale>(pop).unwrap();

        assert_eq!(wallet.credits, 10.0);
        assert!(!morale.modifiers.iter().any(|m| m.label == "Homesick"));
    }

    #[test]
    fn test_failed_remittance_causes_homesick_depression() {
        let mut world = World::new();
        world.init_resource::<RemittanceTracker>();
        let faction = world.spawn_empty().id();

        let pop = world
            .spawn((
                MigrantFamilyInfo {
                    home_faction: faction,
                    remittance_target: 10.0,
                },
                Wallet { credits: 0.0 },
                Morale::default(),
            ))
            .id();

        world.init_resource::<Events<MigrantArrivalEvent>>();
        world.run_system_once(process_remittances_system).unwrap();

        let wallet = world.get::<Wallet>(pop).unwrap();
        let morale = world.get::<Morale>(pop).unwrap();

        assert_eq!(wallet.credits, 0.0);
        assert!(morale.modifiers.iter().any(|m| m.label == "Homesick"));
    }

    #[test]
    fn test_high_remittances_trigger_migrant_arrival() {
        let mut world = World::new();
        world.init_resource::<RemittanceTracker>();
        let faction = world.spawn_empty().id();

        // Spawn enough pops to trigger the migration event (10 * 10 = 100)
        for _ in 0..10 {
            world.spawn((
                MigrantFamilyInfo {
                    home_faction: faction,
                    remittance_target: 10.0,
                },
                Wallet { credits: 20.0 },
                Morale::default(),
            ));
        }

        world.init_resource::<Events<MigrantArrivalEvent>>();
        world.run_system_once(process_remittances_system).unwrap();

        let events = world.resource::<Events<MigrantArrivalEvent>>();
        let mut reader = events.get_cursor();
        let mut count = 0;
        for ev in reader.read(events) {
            assert_eq!(ev.home_faction, faction);
            count += 1;
        }

        assert_eq!(
            count, 1,
            "Should have triggered exactly one migrant arrival event"
        );
    }
}
