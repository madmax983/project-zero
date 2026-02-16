use crate::layer1::farm::Farm;
use crate::layer1::map::GridPosition;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::stress::BreakdownType;
use crate::layer1::structure::Structure;
use crate::layer1::unrest::{MentalBreakType, MentalState};
use crate::layer1::utility_eval_types::PopEvalData;
use crate::layer1::utility_types::{ActionType, manhattan_distance};
use bevy_ecs::prelude::*;

/// Evaluates actions for a pop undergoing a mental break.
pub(crate) fn evaluate_mental_break(
    data: &PopEvalData,
    world: &mut World,
) -> Option<(ActionType, f32, Option<Entity>)> {
    if let Some(breakdown) = data.breakdown {
        let best_utility = 100.0;
        let mut best_target = None;

        let best_action = match breakdown.breakdown_type {
            BreakdownType::Dazing => ActionType::Daze,
            BreakdownType::SadWander => ActionType::SadWander,
            BreakdownType::HideInRoom => ActionType::HideInRoom,
            BreakdownType::BingeEating => {
                find_food_target(data, world, &mut best_target);
                ActionType::Binge
            }
            BreakdownType::FireStarting => {
                find_structure_target(data, world, &mut best_target);
                ActionType::FireStarting
            }
        };
        return Some((best_action, best_utility, best_target));
    }

    let Some(MentalState::Broken(break_type)) = data.mental_state else {
        return None;
    };

    let best_utility = 100.0;
    let mut best_target = None;

    let best_action = match break_type {
        MentalBreakType::Vandalize => {
            find_structure_target(data, world, &mut best_target);
            ActionType::Vandalize
        }
        MentalBreakType::Binge => {
            find_food_target(data, world, &mut best_target);
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

fn find_structure_target(data: &PopEvalData, world: &mut World, best_target: &mut Option<Entity>) {
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
    *best_target = closest_target;
}

fn find_food_target(data: &PopEvalData, world: &mut World, best_target: &mut Option<Entity>) {
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
    *best_target = closest_target;
}
