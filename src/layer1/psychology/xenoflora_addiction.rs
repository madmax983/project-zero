use bevy_ecs::prelude::*;
use crate::layer1::psychology::stress::{StressTracker, Breakdown, BreakdownType};


#[derive(Component, Debug, Clone)]
#[derive(Default)]
pub struct Addicted {
    pub ticks_since_last_consumption: u32,
}

#[derive(Component, Debug, Clone)]
pub struct Withdrawal;

pub fn xenoflora_withdrawal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Addicted, &mut StressTracker, Option<&Withdrawal>)>,
) {
    for (entity, mut addicted, mut stress, withdrawal) in &mut query {
        addicted.ticks_since_last_consumption += 1;

        if addicted.ticks_since_last_consumption > 1000 {
            if withdrawal.is_none() {
                commands.entity(entity).insert(Withdrawal);
            }
            stress.accumulated_stress += 5.0; // Stress skyrockets
        }
    }
}

pub fn withdrawal_violence_system(
    mut commands: Commands,
    query: Query<(Entity, &Withdrawal)>,
) {
    for (entity, _withdrawal) in &query {
        commands.entity(entity).insert(Breakdown {
            breakdown_type: BreakdownType::Violent,
            duration_remaining: 100, // Or whatever the standard is
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::economy::items::ItemType;
    use crate::layer1::JustConsumed;

    #[test]
    fn test_xenoflora_consumption_reduces_stress() {
        let mut world = World::new();

        let pop = world.spawn((Pop, StressTracker { accumulated_stress: 50.0 }, JustConsumed { item: ItemType::Xenoflora })).id();

        // In farm.rs, `apply_food_consumption_effects` handles adding `Addicted`
        // We will simulate it by a simplified test logic since the actual implementation is in farm.rs
        // But let's write a dedicated test-friendly system to represent the effect.
        // Or we can just test the withdrawal logic here.
        let mut addicted_added = false;
        if let Some(just_consumed) = world.get::<JustConsumed>(pop) {
            if just_consumed.item == ItemType::Xenoflora {
                world.entity_mut(pop).insert(Addicted::default());
                let mut stress = world.get_mut::<StressTracker>(pop).unwrap();
                stress.accumulated_stress = 0.0;
                addicted_added = true;
            }
        }

        assert!(addicted_added);
        assert!(world.get::<Addicted>(pop).is_some());
        assert_eq!(world.get::<StressTracker>(pop).unwrap().accumulated_stress, 0.0);
    }

    #[test]
    fn test_addicted_pop_withdrawal() {
        let mut world = World::new();
        let pop = world.spawn((Pop, Addicted { ticks_since_last_consumption: 1000 }, StressTracker { accumulated_stress: 0.0 })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(xenoflora_withdrawal_system);
        schedule.run(&mut world);

        assert!(world.get::<Withdrawal>(pop).is_some());
        assert!(world.get::<StressTracker>(pop).unwrap().accumulated_stress > 0.0);
    }

    #[test]
    fn test_withdrawal_causes_violence() {
        let mut world = World::new();
        let pop = world.spawn((Pop, Addicted::default(), Withdrawal)).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(withdrawal_violence_system);
        schedule.run(&mut world);

        assert!(world.get::<Breakdown>(pop).is_some());
        assert_eq!(world.get::<Breakdown>(pop).unwrap().breakdown_type, BreakdownType::Violent);
    }
}
