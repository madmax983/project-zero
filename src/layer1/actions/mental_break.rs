use crate::layer1::farm::Farm;
use crate::layer1::map::GridPosition;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::structure::Structure;
use crate::layer1::unrest::{MentalBreakType, MentalState};
use crate::layer1::utility_types::{ActionType, PopEvalData, manhattan_distance};
use bevy_ecs::prelude::*;

/// Evaluates actions for a pop undergoing a mental break.
pub fn evaluate_mental_break(
    data: &PopEvalData,
    world: &mut World,
) -> Option<(ActionType, f32, Option<Entity>)> {
    let Some(MentalState::Broken(break_type)) = data.mental_state else {
        return None;
    };

    let best_utility = 100.0;
    let mut best_target = None;

    let best_action = match break_type {
        MentalBreakType::Vandalize => {
            // Find closest structure to destroy
            let mut closest_dist = i32::MAX;
            let mut closest_target = None;

            let mut structures_state = world.query::<(Entity, &GridPosition, &Structure)>();

            for (target_entity, target_pos, _) in structures_state.iter(world) {
                if target_entity == data.entity {
                    continue;
                }
                let dist = manhattan_distance(&data.pos, target_pos);
                if dist < closest_dist {
                    closest_dist = dist;
                    closest_target = Some(target_entity);
                }
            }
            best_target = closest_target;
            ActionType::Vandalize
        }
        MentalBreakType::Binge => {
            // Find closest Stockpile or Farm (Assumed food source)
            let mut closest_dist = i32::MAX;
            let mut closest_target = None;

            let mut stockpiles_state = world.query::<(Entity, &GridPosition, &Stockpile)>();
            for (entity, pos, _) in stockpiles_state.iter(world) {
                let dist = manhattan_distance(&data.pos, pos);
                if dist < closest_dist {
                    closest_dist = dist;
                    closest_target = Some(entity);
                }
            }

            let mut farms_state = world.query::<(
                Entity,
                &GridPosition,
                &Farm,
                Option<&crate::layer1::building::ShiftSchedule>,
            )>();
            for (entity, pos, _, _) in farms_state.iter(world) {
                let dist = manhattan_distance(&data.pos, pos);
                if dist < closest_dist {
                    closest_dist = dist;
                    closest_target = Some(entity);
                }
            }

            best_target = closest_target;
            ActionType::Binge
        }
        MentalBreakType::Daze => ActionType::Daze,
        MentalBreakType::Sleepwalking => {
            // Sleepwalkers just wander. Target is assigned by assign_sleepwalk_target_system.
            best_target = None;
            ActionType::Sleepwalking
        }
    };

    Some((best_action, best_utility, best_target))
}
