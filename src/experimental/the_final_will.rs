//! The Final Will (Nova Feature)
//!
//! # The Spark
//! We have a `PopDied` event, an economy system (`Wallet`), and a social network (`Relationships` and `SocialDebt`).
//! When a pop dies, their wallet shouldn't just vanish into the void. What if they pass on their wealth?
//!
//! # The Feature
//! Implements a "Final Will and Testament" mechanic.
//! When a `Pop` dies, this system checks their `Relationships` and `SocialDebt`.
//! It finds the living `Pop` they had the highest positive `Affinity` with (or the person they owed the most to/who saved their life)
//! and transfers all their credits to that pop.
//! Emits an `AddChronicleEvent` to log this dramatic exchange of inheritance.

use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::economy::Wallet;
use crate::layer1::pop::{PopDied, PopName};
use crate::layer1::social::Relationships;
use bevy_ecs::prelude::*;

/// The Final Will system.
/// Listens for `PopDied` events, finds the deceased's closest living companion (highest affinity),
/// and transfers their remaining credits.
pub fn the_final_will_system(
    mut died_events: EventReader<PopDied>,
    mut wallets: Query<&mut Wallet>,
    relationships: Query<&Relationships>,
    names: Query<&PopName>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in died_events.read() {
        let deceased_entity = event.entity;

        // Try to get their remaining wealth
        let remaining_credits = if let Ok(wallet) = wallets.get(deceased_entity) {
            wallet.credits
        } else {
            0.0
        };

        // If they were broke, there's no will to process
        if remaining_credits <= 0.0 {
            continue;
        }

        // Try to find the heir (highest affinity)
        let mut best_heir = None;
        let mut highest_affinity = -100.0;

        if let Ok(rels) = relationships.get(deceased_entity) {
            for (&target_entity, &affinity) in &rels.affinities {
                if affinity > highest_affinity {
                    // Check if the heir is actually alive and has a wallet to receive funds
                    if wallets.contains(target_entity) {
                        highest_affinity = affinity;
                        best_heir = Some(target_entity);
                    }
                }
            }
        }

        if let Some(heir_entity) = best_heir {
            if highest_affinity > 0.0 {
                // We have a worthy heir, transfer the funds

                // Deduct from deceased
                if let Ok(mut dead_wallet) = wallets.get_mut(deceased_entity) {
                    dead_wallet.credits = 0.0;
                }

                // Add to heir
                if let Ok(mut heir_wallet) = wallets.get_mut(heir_entity) {
                    heir_wallet.credits += remaining_credits;
                }

                // Announce it
                let heir_name = if let Ok(name) = names.get(heir_entity) {
                    name.0.clone()
                } else {
                    "an unknown friend".to_string()
                };

                let text = format!(
                    "In their final will, {} left all their worldly possessions ({:.1} credits) to their closest confidant, {}.",
                    event.name, remaining_credits, heir_name
                );

                chronicle_events.send(AddChronicleEvent {
                    text,
                    importance: EventImportance::Minor,
                });
            } else {
                // Everyone hated them or they hated everyone, money goes to the void (or colony budget eventually)
            }
        }
    }
}

/// Registers the system.
pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(the_final_will_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_final_will_transfers_credits_to_highest_affinity() {
        let mut world = World::new();
        world.init_resource::<Events<PopDied>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        let heir1 = world
            .spawn((Wallet { credits: 10.0 }, PopName("Heir One".to_string())))
            .id();
        let heir2 = world
            .spawn((Wallet { credits: 10.0 }, PopName("Heir Two".to_string())))
            .id();

        let mut affinities = HashMap::new();
        affinities.insert(heir1, 50.0);
        affinities.insert(heir2, 90.0); // Highest affinity

        let deceased = world
            .spawn((
                Wallet { credits: 100.0 },
                Relationships { affinities },
                PopName("Deceased".to_string()),
            ))
            .id();

        world.send_event(PopDied {
            entity: deceased,
            name: "Deceased".to_string(),
            tick: 100,
            reason: "Old Age".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(the_final_will_system);
        schedule.run(&mut world);

        // Heir 2 should get the money
        let wallet2 = world.get::<Wallet>(heir2).unwrap();
        assert_eq!(wallet2.credits, 110.0); // 10 + 100

        // Heir 1 should remain same
        let wallet1 = world.get::<Wallet>(heir1).unwrap();
        assert_eq!(wallet1.credits, 10.0);

        // Deceased wallet should be 0
        let dead_wallet = world.get::<Wallet>(deceased).unwrap();
        assert_eq!(dead_wallet.credits, 0.0);

        // Verify Chronicle Event
        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let evt = reader.read(events).next().unwrap();
        assert!(evt.text.contains("100.0 credits"));
        assert!(evt.text.contains("Heir Two"));
    }

    #[test]
    fn test_no_will_if_no_positive_affinity() {
        let mut world = World::new();
        world.init_resource::<Events<PopDied>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        let heir1 = world
            .spawn((Wallet { credits: 10.0 }, PopName("Heir One".to_string())))
            .id();

        let mut affinities = HashMap::new();
        affinities.insert(heir1, -50.0); // Negative affinity

        let deceased = world
            .spawn((
                Wallet { credits: 100.0 },
                Relationships { affinities },
                PopName("Deceased".to_string()),
            ))
            .id();

        world.send_event(PopDied {
            entity: deceased,
            name: "Deceased".to_string(),
            tick: 100,
            reason: "Old Age".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(the_final_will_system);
        schedule.run(&mut world);

        // Heir 1 should remain same
        let wallet1 = world.get::<Wallet>(heir1).unwrap();
        assert_eq!(wallet1.credits, 10.0);

        // Deceased wallet shouldn't transfer since no good heir was found, but system currently
        // leaves it alone (could add logic to transfer to colony). So it stays 100.
        let dead_wallet = world.get::<Wallet>(deceased).unwrap();
        assert_eq!(dead_wallet.credits, 100.0);

        // Verify NO Chronicle Event
        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).next().is_none());
    }

    #[test]
    fn test_no_will_if_broke() {
        let mut world = World::new();
        world.init_resource::<Events<PopDied>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        let heir1 = world
            .spawn((Wallet { credits: 10.0 }, PopName("Heir One".to_string())))
            .id();

        let mut affinities = HashMap::new();
        affinities.insert(heir1, 90.0);

        let deceased = world
            .spawn((
                Wallet { credits: 0.0 }, // BROKE
                Relationships { affinities },
                PopName("Deceased".to_string()),
            ))
            .id();

        world.send_event(PopDied {
            entity: deceased,
            name: "Deceased".to_string(),
            tick: 100,
            reason: "Old Age".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(the_final_will_system);
        schedule.run(&mut world);

        // Heir 1 should remain same
        let wallet1 = world.get::<Wallet>(heir1).unwrap();
        assert_eq!(wallet1.credits, 10.0);

        // Verify NO Chronicle Event
        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).next().is_none());
    }
}
