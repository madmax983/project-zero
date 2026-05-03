//! Debt Prison
//!
//! Simulates the consequences of extreme colonial debt. When debt becomes insurmountable,
//! a shadow syndicate may offer to clear it—in exchange for offloading their most volatile
//! and spiteful criminals into the colony as new citizens.
use crate::layer1::factions::{FactionId, FactionMember};
use crate::layer1::pop::PopBundle;
use crate::layer1::traits::{Trait, Traits};
use crate::layer2::trade::blockade::ColonyDebt;
use bevy::prelude::*;

#[derive(Event)]
pub struct BailoutOfferEvent;

#[derive(Event)]
pub struct AcceptBailoutEvent;

/// Triggers a bailout offer when a colony's debt reaches critical levels.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer3::events::debt_prison::{check_bailout_condition_system, BailoutOfferEvent};
/// use scale::layer2::trade::blockade::ColonyDebt;
/// let mut app = App::new();
/// app.add_event::<BailoutOfferEvent>();
/// app.world_mut().insert_resource(ColonyDebt { amount: 1_000_000.0, threshold: 50_000.0 });
/// app.add_systems(Update, check_bailout_condition_system);
/// app.update();
/// let events = app.world().resource::<Events<BailoutOfferEvent>>();
/// let mut cursor = events.get_cursor();
/// assert_eq!(cursor.read(events).count(), 1);
/// ```
pub fn check_bailout_condition_system(
    debt: Res<ColonyDebt>,
    mut bailout_events: EventWriter<BailoutOfferEvent>,
) {
    if debt.amount >= debt.threshold && debt.amount >= 1_000_000.0 {
        bailout_events.send(BailoutOfferEvent);
    }
}

pub fn process_bailout_acceptance_system(
    mut commands: Commands,
    mut events: EventReader<AcceptBailoutEvent>,
    mut debt: ResMut<ColonyDebt>,
) {
    for _ in events.read() {
        // Clear debt
        debt.amount = 0.0;

        // Spawn criminals
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let mut bundle = PopBundle::random(0, 0, &mut rng);
            let mut traits = Traits::default();
            traits.add(Trait::Volatile);
            traits.add(Trait::Spiteful);
            bundle.traits = traits;
            bundle.faction = FactionMember {
                faction_id: Some(FactionId::Cartel),
            };
            commands.spawn(bundle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::factions::{FactionId, FactionMember};
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer2::trade::blockade::ColonyDebt;

    #[test]
    fn test_bailout_offer_triggers_on_high_debt() {
        // Arrange
        let mut app = App::new();
        app.add_event::<BailoutOfferEvent>();
        // Set colony debt to a critical level
        app.world_mut().insert_resource(ColonyDebt {
            amount: 1000000.0,
            threshold: 50000.0,
        });

        app.add_systems(Update, check_bailout_condition_system);

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<BailoutOfferEvent>>();
        let reader = events.get_cursor();
        assert_eq!(reader.len(events), 1, "Should trigger a bailout offer");
    }

    #[test]
    fn test_accept_bailout_clears_debt_and_spawns_criminals() {
        // Arrange
        let mut app = App::new();
        app.add_event::<AcceptBailoutEvent>();
        app.world_mut().insert_resource(ColonyDebt {
            amount: 1000000.0,
            threshold: 50000.0,
        });
        app.world_mut().send_event(AcceptBailoutEvent);

        app.add_systems(Update, process_bailout_acceptance_system);

        // Act
        app.update();

        // Assert
        // Verify ColonyDebt is 0.0
        let debt = app.world().resource::<ColonyDebt>();
        assert_eq!(debt.amount, 0.0, "Debt should be cleared");

        // Verify a large number of Pops with criminal traits and high stats are spawned
        let mut query = app.world_mut().query::<(&Pop, &Traits)>();
        let pop_count = query.iter(app.world()).count();
        assert_eq!(pop_count, 100, "Should spawn exactly 100 criminal pops");

        for (_, traits) in query.iter(app.world()) {
            assert!(
                traits.has(Trait::Volatile),
                "Criminals should have the Volatile trait"
            );
            assert!(
                traits.has(Trait::Spiteful),
                "Criminals should have the Spiteful trait"
            );
        }
    }

    #[test]
    fn test_criminal_faction_formation() {
        // Arrange
        let mut app = App::new();
        app.add_event::<AcceptBailoutEvent>();
        app.world_mut().insert_resource(ColonyDebt {
            amount: 1000000.0,
            threshold: 50000.0,
        });
        app.world_mut().send_event(AcceptBailoutEvent);

        app.add_systems(Update, process_bailout_acceptance_system);

        // Act
        app.update();

        // Assert
        // Verify a criminal faction exists and the new pops belong to it
        let mut query = app.world_mut().query::<(&Pop, &FactionMember)>();
        let pop_count = query.iter(app.world()).count();
        assert_eq!(
            pop_count, 100,
            "Should spawn exactly 100 criminal pops with factions"
        );

        for (_, faction_member) in query.iter(app.world()) {
            assert_eq!(
                faction_member.faction_id,
                Some(FactionId::Cartel),
                "Criminals should belong to the Cartel faction"
            );
        }
    }
}
