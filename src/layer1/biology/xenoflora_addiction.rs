use bevy_ecs::prelude::*;
use crate::layer1::economy::inventory::Inventory;
use crate::layer1::items::ItemType;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::social::unrest::{MentalBreakType, MentalState};

/// Component indicating a Pop is addicted to Xenoflora.
#[derive(Component, Default, Debug, Clone)]
pub struct Addicted {
    pub ticks_since_last_consumption: u32,
}

/// Component indicating an addicted Pop is experiencing withdrawal.
#[derive(Component, Default, Debug, Clone)]
pub struct Withdrawal;

/// Pop consumes Xenoflora, lowering stress and gaining/resetting addiction.
pub fn xenoflora_consumption_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Inventory,
        &mut StressTracker,
        Option<&mut Addicted>,
    ), With<Pop>>,
) {
    for (entity, mut inventory, mut stress, addicted_opt) in query.iter_mut() {
        let xenoflora_idx = inventory
            .items
            .iter()
            .position(|item| item.item_type == ItemType::Xenoflora);

        if let Some(idx) = xenoflora_idx {
            // Found Xenoflora, consume it if stress is somewhat high or addicted
            if stress.accumulated_stress > 20.0 || addicted_opt.is_some() {
                inventory.items.remove(idx);
                // Reduce stress significantly
                stress.accumulated_stress = (stress.accumulated_stress - 30.0).max(0.0);

                if let Some(mut addicted) = addicted_opt {
                    addicted.ticks_since_last_consumption = 0;
                    commands.entity(entity).remove::<Withdrawal>();
                } else {
                    commands.entity(entity).insert(Addicted {
                        ticks_since_last_consumption: 0,
                    });
                }
            }
        }
    }
}

/// System to handle addiction progression into withdrawal.
pub fn addiction_withdrawal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Addicted, &mut StressTracker, Option<&Withdrawal>), With<Pop>>,
) {
    for (entity, mut addicted, mut stress, withdrawal_opt) in query.iter_mut() {
        addicted.ticks_since_last_consumption += 1;

        if addicted.ticks_since_last_consumption > 100 {
            if withdrawal_opt.is_none() {
                commands.entity(entity).insert(Withdrawal);
            }
            // Increase stress extremely rapidly during withdrawal
            stress.accumulated_stress = (stress.accumulated_stress + 5.0).min(100.0);
        }
    }
}

type WithdrawalViolenceQueryData<'a> = (
    &'a StressTracker,
    &'a mut MentalState,
);

type WithdrawalViolenceQueryFilter = (With<Pop>, With<Withdrawal>);

/// System forcing a violent mental break during withdrawal when stress is maxed.
pub fn withdrawal_causes_violence_system(
    mut query: Query<WithdrawalViolenceQueryData, WithdrawalViolenceQueryFilter>,
) {
    for (stress, mut mental_state) in query.iter_mut() {
        if stress.accumulated_stress >= 100.0 && !matches!(*mental_state, MentalState::Broken(_)) {
            *mental_state = MentalState::Broken(MentalBreakType::Violent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::inventory::InventoryItem;

    #[test]
    fn test_xenoflora_consumption_reduces_stress() {
        let mut world = World::new();

        let mut inventory = Inventory::default();
        inventory.items.push(InventoryItem {
            item_type: ItemType::Xenoflora,
            entity: None,
        });

        let pop = world
            .spawn((
                Pop,
                inventory,
                StressTracker {
                    accumulated_stress: 50.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(xenoflora_consumption_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert_eq!(stress.accumulated_stress, 20.0, "Stress should be reduced");

        assert!(
            world.get::<Addicted>(pop).is_some(),
            "Pop should gain Addicted component"
        );

        let inventory = world.get::<Inventory>(pop).unwrap();
        assert!(
            inventory.items.is_empty(),
            "Xenoflora should be consumed from inventory"
        );
    }

    #[test]
    fn test_addicted_pop_withdrawal() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                Addicted {
                    ticks_since_last_consumption: 100, // One tick away from withdrawal
                },
                StressTracker {
                    accumulated_stress: 50.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(addiction_withdrawal_system);
        schedule.run(&mut world);

        assert!(
            world.get::<Withdrawal>(pop).is_some(),
            "Pop should gain Withdrawal component"
        );

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert_eq!(
            stress.accumulated_stress, 55.0,
            "Stress should rapidly increase during withdrawal"
        );
    }

    #[test]
    fn test_withdrawal_causes_violence() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                Withdrawal,
                StressTracker {
                    accumulated_stress: 100.0,
                },
                MentalState::Normal,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(withdrawal_causes_violence_system);
        schedule.run(&mut world);

        let state = world.get::<MentalState>(pop).unwrap();
        assert!(
            matches!(state, MentalState::Broken(MentalBreakType::Violent)),
            "Pop should enter Violent mental break"
        );
    }
}
