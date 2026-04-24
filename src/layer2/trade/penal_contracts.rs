use crate::layer1::health::Health;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ColonyFunds(pub u32);

#[derive(Component)]
pub struct PenalContract {
    pub payment_per_day: u32,
    pub prisoner_count: u32,
}

#[derive(Component)]
pub struct ContractTimer(pub u32);

#[derive(Component)]
pub struct PrisonerOf(pub Entity);

#[derive(Component)]
pub struct FactionRelation(pub i32);

/// Emitted when a state prisoner dies (INT-539).
#[derive(Event)]
pub struct PrisonerDiedEvent(pub Entity);

#[allow(clippy::needless_pass_by_value)]
pub fn process_penal_contracts_system(
    mut contracts: Query<(&PenalContract, &mut ContractTimer)>,
    mut colonies: Query<&mut ColonyFunds>,
) {
    for (contract, mut timer) in contracts.iter_mut() {
        if timer.0 > 0 {
            timer.0 -= 1;
        }
        if timer.0 == 0 {
            if let Ok(mut funds) = colonies.get_single_mut() {
                funds.0 += contract.payment_per_day * contract.prisoner_count;
            }
            timer.0 = 250; // Standard ticks per day
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn check_prisoner_status_system(
    mut commands: Commands,
    prisoners: Query<(Entity, &PrisonerOf, &Health), With<Pop>>,
    mut factions: Query<&mut FactionRelation>,
    mut death_events: EventWriter<PrisonerDiedEvent>,
) {
    for (entity, prisoner_of, health) in prisoners.iter() {
        if health.current <= 0.0 {
            if let Ok(mut relation) = factions.get_mut(prisoner_of.0) {
                relation.0 -= 50; // Heavy penalty
            }
            // Emit event for INT-539 (Chronicle bridge)
            death_events.send(PrisonerDiedEvent(entity));

            // Remove PrisonerOf so penalty isn't applied infinitely
            commands.entity(entity).remove::<PrisonerOf>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_penal_contract_payment() {
        // Arrange
        let mut world = World::new();
        let colony = world.spawn(ColonyFunds(0)).id();
        let _contract = world
            .spawn((
                PenalContract {
                    payment_per_day: 10,
                    prisoner_count: 5,
                },
                ContractTimer(1),
            ))
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(process_penal_contracts_system);
        schedule.run(&mut world);

        // Assert
        let funds = world.get::<ColonyFunds>(colony).unwrap();
        assert_eq!(
            funds.0, 50,
            "Colony should receive 50 credits (5 prisoners * 10/day)"
        );
    }

    #[test]
    fn test_prisoner_death_penalty() {
        // Arrange
        let mut world = World::new();
        let faction = world.spawn(FactionRelation(100)).id();
        let prisoner = world
            .spawn((
                Pop,
                PrisonerOf(faction),
                Health {
                    current: 0.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        // Act
        world.init_resource::<Events<PrisonerDiedEvent>>();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_prisoner_status_system);
        schedule.run(&mut world);

        // Assert
        let relation = world.get::<FactionRelation>(faction).unwrap();
        assert!(
            relation.0 < 100,
            "Faction relation should drop if prisoner dies"
        );

        // Ensure PrisonerOf is removed so penalty is not applied infinitely
        assert!(world.get::<PrisonerOf>(prisoner).is_none());

        // Running it again should not decrease the relation further
        let relation_after_first_run = relation.0;
        schedule.run(&mut world);
        let relation_after_second_run = world.get::<FactionRelation>(faction).unwrap().0;
        assert_eq!(relation_after_first_run, relation_after_second_run);
    }
}
