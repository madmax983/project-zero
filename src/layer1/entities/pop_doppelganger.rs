use crate::layer1::execution::MovementTarget;
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Pop, PopName};
use crate::layer1::resources::MiningProgress;
use bevy_ecs::prelude::*;

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
    let mut name = None;
    let mut pos = None;
    let mut health = None;

    if let Ok(entity_ref) = world.get_entity(target) {
        if let Some(n) = entity_ref.get::<PopName>() {
            name = Some(n.clone());
        }
        if let Some(p) = entity_ref.get::<GridPosition>() {
            pos = Some(*p);
        }
        if let Some(h) = entity_ref.get::<Health>() {
            health = Some(*h);
        }
    }

    world.despawn(target);

    let mut spawn = world.spawn(Pop);

    if let Some(n) = &name {
        spawn.insert(n.clone());
    }
    if let Some(p) = pos {
        spawn.insert(p);
    }
    if let Some(h) = health {
        spawn.insert(h);
    }

    let orig_name = name
        .map(|n| n.0.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    spawn.insert(Mimic {
        state: MimicState::Hidden,
        original_identity: orig_name,
    });

    spawn.id()
}

pub fn sabotage_system(
    query: Query<(&Mimic, &MovementTarget)>,
    mut target_query: Query<&mut MiningProgress>,
) {
    for (mimic, target) in query.iter() {
        if mimic.state == MimicState::Hidden {
            if let Ok(mut progress) = target_query.get_mut(target.target_entity) {
                // Stall mining progress
                if progress.current > 80.0 {
                    progress.current = 80.0;
                }
            }
        }
    }
}

pub fn reveal_mimic(world: &mut World, target: Entity) -> bool {
    if let Ok(mut entity_mut) = world.get_entity_mut(target) {
        if let Some(mut mimic) = entity_mut.get_mut::<Mimic>() {
            mimic.state = MimicState::Revealed;
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::execution::AtTarget;
    use crate::layer1::utility_types::ActionType;

    #[test]
    fn test_mimic_replacement() {
        let mut world = World::new();
        let original_pop = world
            .spawn((
                Pop,
                PopName("Miner Bob".to_string()),
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        let mimic_entity = replace_pop_with_mimic(&mut world, original_pop);

        assert!(world.get_entity(original_pop).is_err());

        let mimic_name = world.get::<PopName>(mimic_entity).unwrap();
        assert_eq!(mimic_name.0, "Miner Bob");

        assert!(world.get::<Mimic>(mimic_entity).is_some());
    }

    #[test]
    fn test_mimic_sabotage_during_work() {
        let mut world = World::new();

        let rock = world
            .spawn(MiningProgress {
                current: 90.0,
                max: 100.0,
            })
            .id();

        world.spawn((
            Pop,
            Mimic {
                state: MimicState::Hidden,
                original_identity: "Miner Bob".to_string(),
            },
            MovementTarget {
                target_entity: rock,
                target_position: GridPosition { x: 1, y: 1 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(sabotage_system);
        schedule.run(&mut world);

        let progress = world.get::<MiningProgress>(rock).unwrap();
        assert!(progress.current <= 80.0);
    }

    #[test]
    fn test_mimic_detection_via_scan() {
        let mut world = World::new();
        let mimic = world
            .spawn((
                Pop,
                Mimic {
                    state: MimicState::Hidden,
                    original_identity: "Suspect".to_string(),
                },
                PopName("Suspect".to_string()),
            ))
            .id();

        let is_mimic = reveal_mimic(&mut world, mimic);
        assert!(is_mimic);

        let mimic_comp = world.get::<Mimic>(mimic).unwrap();
        assert_eq!(mimic_comp.state, MimicState::Revealed);
    }
}
