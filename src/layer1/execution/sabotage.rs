use crate::layer1::architecture::Structure;
use crate::layer1::execution::components::MovementTarget;
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;

pub fn sabotage_action_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut PopAction, &MovementTarget)>,
    mut structures: Query<&mut Structure>,
) {
    for (pop_entity, mut action, mt) in pops.iter_mut() {
        if action.current == ActionType::Sabotage {
            let target = mt.target_entity;
            if let Ok(mut structure) = structures.get_mut(target) {
                structure.current_hp -= 10.0;
                if structure.current_hp < 0.0 {
                    structure.current_hp = 0.0;
                }
            }
            action.current = ActionType::Idle;
            commands.entity(pop_entity).remove::<MovementTarget>();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::architecture::Structure;
    use crate::layer1::execution::components::MovementTarget;
    use crate::layer1::execution::sabotage::sabotage_action_system;
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_types::{ActionType, PopAction};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_sabotage_action_system() {
        let mut world = World::new();

        let mut schedule = Schedule::default();
        schedule.add_systems(sabotage_action_system);

        let structure_entity = world
            .spawn(Structure {
                current_hp: 50.0,
                max_hp: 100.0,
            })
            .id();

        let pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::Sabotage,
                    current_utility: 1.0,
                    ticks_committed: 1,
                },
                MovementTarget {
                    target_entity: structure_entity,
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::Sabotage,
                },
            ))
            .id();

        schedule.run(&mut world);

        assert_eq!(
            world.get::<Structure>(structure_entity).unwrap().current_hp,
            40.0
        );
        assert_eq!(
            world.get::<PopAction>(pop_entity).unwrap().current,
            ActionType::Idle
        );
        assert!(world.get::<MovementTarget>(pop_entity).is_none());
    }

    #[test]
    fn test_sabotage_action_system_underflow() {
        let mut world = World::new();

        let mut schedule = Schedule::default();
        schedule.add_systems(sabotage_action_system);

        let structure_entity = world
            .spawn(Structure {
                current_hp: 5.0,
                max_hp: 100.0,
            })
            .id();

        let _pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::Sabotage,
                    current_utility: 1.0,
                    ticks_committed: 1,
                },
                MovementTarget {
                    target_entity: structure_entity,
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::Sabotage,
                },
            ))
            .id();

        schedule.run(&mut world);

        assert_eq!(
            world.get::<Structure>(structure_entity).unwrap().current_hp,
            0.0
        );
    }

    #[test]
    fn test_sabotage_action_system_ignores_other_actions() {
        let mut world = World::new();

        let mut schedule = Schedule::default();
        schedule.add_systems(sabotage_action_system);

        let structure_entity = world
            .spawn(Structure {
                current_hp: 50.0,
                max_hp: 100.0,
            })
            .id();

        let pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::Idle, // Not Sabotage
                    current_utility: 1.0,
                    ticks_committed: 1,
                },
                MovementTarget {
                    target_entity: structure_entity,
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::Idle,
                },
            ))
            .id();

        schedule.run(&mut world);

        assert_eq!(
            world.get::<Structure>(structure_entity).unwrap().current_hp,
            50.0
        ); // Should not change
        assert_eq!(
            world.get::<PopAction>(pop_entity).unwrap().current,
            ActionType::Idle
        );
        assert!(world.get::<MovementTarget>(pop_entity).is_some()); // Should not remove
    }

    #[test]
    fn test_sabotage_action_system_missing_structure() {
        let mut world = World::new();

        let mut schedule = Schedule::default();
        schedule.add_systems(sabotage_action_system);

        let missing_structure_entity = Entity::from_raw(999);

        let pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::Sabotage,
                    current_utility: 1.0,
                    ticks_committed: 1,
                },
                MovementTarget {
                    target_entity: missing_structure_entity, // Does not exist
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::Sabotage,
                },
            ))
            .id();

        schedule.run(&mut world);

        // Should still become Idle and remove MovementTarget
        assert_eq!(
            world.get::<PopAction>(pop_entity).unwrap().current,
            ActionType::Idle
        );
        assert!(world.get::<MovementTarget>(pop_entity).is_none());
    }
}
