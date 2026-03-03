use bevy_ecs::prelude::*;
use crate::layer1::stress::BreakdownType;
use crate::layer1::unrest::{MentalBreakType, MentalState};
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::manhattan_distance;
use crate::layer1::utility_types::ActionType;

/// Evaluates actions for a pop undergoing a mental break.
pub(crate) fn evaluate_mental_break(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> Option<(ActionType, f32, Option<Entity>)> {
    if let Some(breakdown) = data.breakdown {
        let best_utility = 100.0;
        let mut best_target = None;

        let best_action = match breakdown.breakdown_type {
            BreakdownType::Dazing => ActionType::Daze,
            BreakdownType::SadWander => ActionType::SadWander,
            BreakdownType::HideInRoom => ActionType::HideInRoom,
            BreakdownType::BingeEating => {
                find_food_target(data, buffer, &mut best_target);
                ActionType::Binge
            }
            BreakdownType::FireStarting => {
                find_structure_target(data, buffer, &mut best_target);
                ActionType::FireStarting
            }
        };
        return Some((best_action, best_utility, best_target));
    }

    if let Some(MentalState::Broken(break_type)) = data.mental_state {
        let best_utility = 100.0;
        let mut best_target = None;

        let best_action = match break_type {
            MentalBreakType::Vandalize => {
                find_structure_target(data, buffer, &mut best_target);
                ActionType::Vandalize
            }
            MentalBreakType::Binge => {
                find_food_target(data, buffer, &mut best_target);
                ActionType::Binge
            }
            MentalBreakType::Daze => ActionType::Daze,
            MentalBreakType::Sleepwalking => {
                // Sleepwalkers just wander. Target is assigned by assign_sleepwalk_target_system.
                best_target = None;
                ActionType::Sleepwalking
            }
        };

        return Some((best_action, best_utility, best_target));
    }

    None
}

fn find_structure_target(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
    best_target: &mut Option<Entity>,
) {
    let mut closest_dist = i32::MAX;
    let mut closest_target = None;

    for structure in &buffer.all_structures {
        if structure.entity == data.entity {
            continue;
        }
        let dist = manhattan_distance(&data.pos, &structure.pos);
        if dist < closest_dist {
            closest_dist = dist;
            closest_target = Some(structure.entity);
        }
    }
    *best_target = closest_target;
}

fn find_food_target(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
    best_target: &mut Option<Entity>,
) {
    let mut closest_dist = i32::MAX;
    let mut closest_target = None;

    for stockpile in &buffer.stockpiles {
        let dist = manhattan_distance(&data.pos, &stockpile.pos);
        if dist < closest_dist {
            closest_dist = dist;
            closest_target = Some(stockpile.entity);
        }
    }

    for farm in &buffer.farms {
        let dist = manhattan_distance(&data.pos, &farm.pos);
        if dist < closest_dist {
            closest_dist = dist;
            closest_target = Some(farm.entity);
        }
    }
    *best_target = closest_target;
}
