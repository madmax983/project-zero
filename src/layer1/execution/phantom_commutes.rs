use crate::layer1::execution::components::{HabituatedRoute, MovementTarget};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

/// System to override standard pathfinding with habituated routes.
pub fn apply_phantom_commute_system(
    mut q_pops: Query<(&mut MovementTarget, &mut HabituatedRoute, &GridPosition), With<Pop>>,
) {
    for (mut target, mut route, pos) in q_pops.iter_mut() {
        // Track frustration if stuck
        if let Some(last_p) = route.last_pos {
            if last_p == *pos {
                route.frustration += 1;
            } else {
                route.frustration = 0;
            }
        }
        route.last_pos = Some(*pos);

        // Find current step in path and override movement target to the next step
        if let Some(current_idx) = route.path.iter().position(|p| *p == *pos) {
            if current_idx + 1 < route.path.len() {
                target.target_position = route.path[current_idx + 1];
            }
        } else {
            // If completely off path, try to go to the closest point or the start.
            // For MVP, just go to the first step.
            if let Some(first) = route.path.first() {
                target.target_position = *first;
            }
        }
    }
}

/// System to remove completed or overly frustrating habituated routes.
pub fn remove_resolved_phantom_commute_system(
    mut commands: Commands,
    q_pops: Query<(Entity, &HabituatedRoute, &GridPosition), With<Pop>>,
) {
    for (entity, route, pos) in q_pops.iter() {
        let reached_end = route.path.last().is_some_and(|last| *last == *pos);
        if reached_end || route.frustration > 10 {
            commands.entity(entity).remove::<HabituatedRoute>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::utility_types::ActionType;

    #[test]
    fn test_pop_follows_phantom_commute_ignoring_hazards() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, apply_phantom_commute_system);

        let start = GridPosition { x: 0, y: 0 };
        let next = GridPosition { x: 1, y: 0 };

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                start,
                MovementTarget {
                    target_entity: Entity::PLACEHOLDER,
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::Work,
                },
                HabituatedRoute {
                    path: vec![start, next],
                    urgency: 1.0,
                    frustration: 0,
                    last_pos: None,
                },
            ))
            .id();

        app.update();

        let target = app.world().get::<MovementTarget>(pop).unwrap();
        assert_eq!(
            target.target_position, next,
            "Pop should target the next step in the habituated route."
        );
    }

    #[test]
    fn test_pop_completes_phantom_commute() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, remove_resolved_phantom_commute_system);

        let pos = GridPosition { x: 2, y: 2 };

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                pos,
                HabituatedRoute {
                    path: vec![GridPosition { x: 1, y: 1 }, pos],
                    urgency: 1.0,
                    frustration: 0,
                    last_pos: None,
                },
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<HabituatedRoute>(pop).is_none(),
            "Habituated route should be removed once the final destination is reached."
        );
    }

    #[test]
    fn test_frustrated_phantom_commute_removed() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, remove_resolved_phantom_commute_system);

        let pos = GridPosition { x: 0, y: 0 };

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                pos,
                HabituatedRoute {
                    path: vec![pos, GridPosition { x: 5, y: 5 }], // Not at end
                    urgency: 1.0,
                    frustration: 11, // High frustration
                    last_pos: None,
                },
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<HabituatedRoute>(pop).is_none(),
            "Habituated route should be removed when frustration is too high."
        );
    }
}
