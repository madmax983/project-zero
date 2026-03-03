use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, PopName};
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::MiningProgress;
use crate::layer1::utility_types::{PopAction, ActionType};
use crate::layer1::execution::components::MovementTarget;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct Mimic {
    pub state: MimicState,
    pub original_identity: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MimicState {
    Hidden,
    Revealed,
}

pub fn replace_pop_with_mimic(world: &mut World, target: Entity) -> Entity {
    let name = world.get::<PopName>(target).unwrap().0.clone();
    let pos = *world.get::<GridPosition>(target).unwrap();
    let health = *world.get::<Health>(target).unwrap();

    world.despawn(target);

    world.spawn((
        Pop,
        PopName(name.clone()),
        pos,
        health,
        Mimic {
            state: MimicState::Hidden,
            original_identity: name,
        },
    )).id()
}

pub fn sabotage_system(
    query: Query<(&PopAction, &MovementTarget, &Mimic)>,
    mut progress_query: Query<&mut MiningProgress>,
) {
    for (action, target, mimic) in query.iter() {
        if mimic.state == MimicState::Hidden {
            if let ActionType::Work = action.current {
                if let Ok(mut progress) = progress_query.get_mut(target.target_entity) {
                    // Sabotage! Reduce progress
                    if progress.current > 0.0 {
                        progress.current = (progress.current - 1.0).max(0.0);
                    }
                }
            }
        }
    }
}

pub fn reveal_mimic(world: &mut World, target: Entity) -> bool {
    if let Some(mut mimic) = world.get_mut::<Mimic>(target) {
        mimic.state = MimicState::Revealed;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, PopName};
    use crate::layer1::health::Health;
    use crate::layer1::doppelganger::{Mimic, sabotage_system, reveal_mimic, MimicState};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::MiningProgress;
    use crate::layer1::utility_types::{PopAction, ActionType};
    use crate::layer1::execution::components::MovementTarget;

    #[test]
    fn test_mimic_replacement() {
        let mut world = World::new();
        let original_pop = world.spawn((
            Pop,
            PopName("Miner Bob".to_string()),
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 10, y: 10 }
        )).id();

        let mimic_entity = crate::layer1::doppelganger::replace_pop_with_mimic(&mut world, original_pop);

        assert!(world.get_entity(original_pop).is_err());

        let mimic_name = world.get::<PopName>(mimic_entity).unwrap();
        assert_eq!(mimic_name.0, "Miner Bob");

        assert!(world.get::<Mimic>(mimic_entity).is_some());
    }

    #[test]
    fn test_mimic_sabotage_during_work() {
        let mut world = World::new();

        let target = world.spawn(MiningProgress {
            current: 95.0,
            max: 100.0,
        }).id();

        let _mimic = world.spawn((
            Pop,
            Mimic { state: MimicState::Hidden, original_identity: "Miner Bob".to_string() },
            PopAction {
                current: ActionType::Work,
                current_utility: 10.0,
                ticks_committed: 1,
            },
            MovementTarget {
                target_entity: target,
                target_position: GridPosition { x: 0, y: 0 },
                for_action: ActionType::Work,
            }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(sabotage_system);
        schedule.run(&mut world);

        let progress = world.get::<MiningProgress>(target).unwrap();
        assert!(progress.current < 95.0);
    }

    #[test]
    fn test_mimic_detection_via_scan() {
        let mut world = World::new();
        let mimic = world.spawn((
            Pop,
            Mimic { state: MimicState::Hidden, original_identity: "Suspect".to_string() },
            PopName("Suspect".to_string()),
        )).id();

        let is_mimic = reveal_mimic(&mut world, mimic);

        assert!(is_mimic);

        let mimic_comp = world.get::<Mimic>(mimic).unwrap();
        assert_eq!(mimic_comp.state, MimicState::Revealed);
    }
}
