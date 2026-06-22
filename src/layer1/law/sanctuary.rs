use crate::layer1::execution::components::MovementTarget;
use crate::layer1::law::justice::Wanted;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::utility_types::ActionType;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;

pub fn wanted_pop_migration_scoring_system(
    mut commands: Commands,
    wanted_query: Query<(Entity, &GridPosition, &Wanted), With<Pop>>,
    zone_grid: Res<ZoneGrid>,
) {
    let mut sanctuary_pos = None;
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    for y in 0..zone_grid.height {
        for x in 0..zone_grid.width {
            if zone_grid.get(x as i32, y as i32) == ZoneType::Sanctuary {
                sanctuary_pos = Some(GridPosition {
                    x: x as i32,
                    y: y as i32,
                });
                break;
            }
        }
        if sanctuary_pos.is_some() {
            break;
        }
    }

    if let Some(sanctuary) = sanctuary_pos {
        for (pop_entity, current_pos, _wanted) in wanted_query.iter() {
            if zone_grid.get(current_pos.x, current_pos.y) != ZoneType::Sanctuary {
                // MVP: Force a move command
                commands.entity(pop_entity).insert(MovementTarget {
                    target_entity: pop_entity, // Self as placeholder
                    target_position: sanctuary,
                    for_action: ActionType::Flee,
                });
            }
        }
    }
}

pub fn filter_police_targets_system(
    mut commands: Commands,
    warden_query: Query<(Entity, &MovementTarget)>,
    zone_grid: Res<ZoneGrid>,
) {
    for (entity, target) in warden_query.iter() {
        if target.for_action == ActionType::Warden
            && zone_grid.get(target.target_position.x, target.target_position.y)
                == ZoneType::Sanctuary
        {
            commands.entity(entity).remove::<MovementTarget>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_wanted_pops_migrate_to_sanctuary_zones() {
        let mut world = World::new();

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Sanctuary); // Sanctuary at 5,5
        world.insert_resource(zone_grid);

        let wanted_pop = world
            .spawn((Pop, GridPosition { x: 0, y: 0 }, Wanted { severity: 1.0 }))
            .id();

        world
            .run_system_once(wanted_pop_migration_scoring_system)
            .unwrap();

        let target = world.get::<MovementTarget>(wanted_pop);
        assert!(
            target.is_some(),
            "Wanted Pops should generate high-priority move actions towards Sanctuary zones."
        );

        let target = target.unwrap();
        assert_eq!(target.target_position.x, 5);
        assert_eq!(target.target_position.y, 5);
        assert_eq!(target.for_action, ActionType::Flee);
    }

    #[test]
    fn test_police_cannot_target_wanted_pops_in_sanctuary() {
        let mut world = World::new();

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Sanctuary);
        world.insert_resource(zone_grid);

        let police = world
            .spawn(MovementTarget {
                target_entity: Entity::PLACEHOLDER,
                target_position: GridPosition { x: 5, y: 5 }, // Targeting Sanctuary
                for_action: ActionType::Warden,
            })
            .id();

        world.run_system_once(filter_police_targets_system).unwrap();

        assert!(
            world.get::<MovementTarget>(police).is_none(),
            "Police should not be able to target or arrest Pops inside a Sanctuary Zone."
        );
    }
}
