use crate::layer1::social::factions::FactionId;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Component indicating a pop has family off-world they support.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct MigrantFamilyInfo {
    /// The faction the family belongs to.
    pub home_faction: FactionId,
    /// The amount of credits the pop tries to send per tick.
    pub remittance_target: f32,
}

impl Default for MigrantFamilyInfo {
    fn default() -> Self {
        Self {
            home_faction: FactionId::Unaligned,
            remittance_target: 1.0,
        }
    }
}

/// Event triggered when a faction receives enough remittances to send new migrants.
#[derive(Debug, Clone, PartialEq, Event)]
pub struct MigrantArrivalEvent {
    pub faction: FactionId,
}

/// Resource tracking total remittances sent to each faction.
#[derive(Resource, Debug, Clone, Default)]
pub struct RemittanceLedger {
    pub totals: HashMap<FactionId, f32>,
}

use crate::layer1::economy::Wallet;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer1::traits::{Trait, Traits};

/// System to deduct remittance target from pop's wallet and send to their home faction.
pub fn process_remittances_system(
    mut query: Query<(
        &MigrantFamilyInfo,
        &mut Wallet,
        &mut Morale,
        Option<&Traits>,
    )>,
    mut ledger: ResMut<RemittanceLedger>,
) {
    for (info, mut wallet, mut morale, traits) in &mut query {
        // Only pops with Family trait send remittances
        if let Some(t) = traits {
            if t.has(Trait::Family) {
                if wallet.credits >= info.remittance_target {
                    wallet.credits -= info.remittance_target;
                    let total = ledger.totals.entry(info.home_faction).or_insert(0.0);
                    *total += info.remittance_target;
                } else {
                    morale.add_modifier(MoodModifier {
                        label: "Homesick".to_string(),
                        value: -0.2,
                        duration: 1000,
                    });
                }
            }
        }
    }
}

/// System to trigger MigrantArrivalEvent when a faction accumulates enough remittances.
pub fn trigger_migrant_arrival_system(
    mut ledger: ResMut<RemittanceLedger>,
    mut events: EventWriter<MigrantArrivalEvent>,
) {
    for (&faction, total) in &mut ledger.totals {
        if *total >= 100.0 {
            events.send(MigrantArrivalEvent { faction });
            *total -= 100.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::Wallet;
    use crate::layer1::social::morale::Morale;
    use crate::layer1::traits::{Trait, Traits};
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(RemittanceLedger::default());
        world.init_resource::<Events<MigrantArrivalEvent>>();
        world
    }

    #[test]
    fn test_pop_sends_remittance_and_maintains_morale() {
        // Arrange: Migrant pop with sufficient personal wealth/resources
        let mut world = setup_world();

        let pop = world
            .spawn((
                MigrantFamilyInfo {
                    home_faction: FactionId::FarmersGuild,
                    remittance_target: 5.0,
                },
                Wallet { credits: 20.0 },
                Morale::default(),
                Traits(1 << (Trait::Family as u8)),
            ))
            .id();

        // Act: Process remittance cycle
        world.run_system_once(process_remittances_system).unwrap();

        // Assert: Pop loses a percentage of wealth, 'Homesick' need remains satisfied
        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 15.0);

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(
            !morale.modifiers.iter().any(|m| m.label == "Homesick"),
            "Should not have Homesick modifier"
        );

        let ledger = world.resource::<RemittanceLedger>();
        assert_eq!(
            ledger
                .totals
                .get(&FactionId::FarmersGuild)
                .copied()
                .unwrap_or(0.0),
            5.0
        );
    }

    #[test]
    fn test_failed_remittance_causes_homesick_depression() {
        // Arrange: Migrant pop with 0 wealth
        let mut world = setup_world();

        let pop = world
            .spawn((
                MigrantFamilyInfo {
                    home_faction: FactionId::FarmersGuild,
                    remittance_target: 5.0,
                },
                Wallet { credits: 2.0 }, // Not enough!
                Morale::default(),
                Traits(1 << (Trait::Family as u8)),
            ))
            .id();

        // Act: Process remittance cycle
        world.run_system_once(process_remittances_system).unwrap();

        // Assert: Pop fails to send remittance, gains 'Homesick' depression mood modifier
        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(
            wallet.credits, 2.0,
            "Should not deduct credits if insufficient"
        );

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(
            morale
                .modifiers
                .iter()
                .any(|m| m.label == "Homesick" && m.value < 0.0),
            "Should have negative Homesick modifier"
        );

        let ledger = world.resource::<RemittanceLedger>();
        assert_eq!(
            ledger
                .totals
                .get(&FactionId::FarmersGuild)
                .copied()
                .unwrap_or(0.0),
            0.0
        );
    }

    #[test]
    fn test_high_remittances_trigger_migrant_arrival() {
        // Arrange: High total volume of remittances sent to a specific faction
        let mut world = setup_world();
        let mut ledger = world.resource_mut::<RemittanceLedger>();
        ledger.totals.insert(FactionId::MinersGuild, 150.0);

        // Act: Process diplomatic/migration triggers
        world
            .run_system_once(trigger_migrant_arrival_system)
            .unwrap();

        // Assert: A new `MigrantArrivalEvent` is fired from the destination faction
        let events = world.resource::<Events<MigrantArrivalEvent>>();
        let mut reader = events.get_cursor();
        let arrivals: Vec<_> = reader.read(events).collect();

        assert_eq!(arrivals.len(), 1);
        assert_eq!(arrivals[0].faction, FactionId::MinersGuild);

        // Check ledger reset
        let ledger = world.resource::<RemittanceLedger>();
        assert_eq!(
            ledger
                .totals
                .get(&FactionId::MinersGuild)
                .copied()
                .unwrap_or(0.0),
            50.0
        ); // 150 - 100
    }
}
