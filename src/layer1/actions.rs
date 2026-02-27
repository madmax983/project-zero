//! # The Action System
//!
//! Consolidated action evaluation logic. Replaces the old `src/layer1/actions/` directory.

#![allow(clippy::trivially_copy_pass_by_ref, clippy::too_many_arguments)]
use crate::layer1::hygiene::SHOWER_WATER_COST;
use crate::layer1::items::Equipment;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::resources::ColonyResources;
use crate::layer1::stress::BreakdownType;
use crate::layer1::traits::Trait;
use crate::layer1::unrest::{MentalBreakType, MentalState};
use crate::layer1::utility_eval_types::{
    PopEvalData, ScorableCandidate, UtilityAIBuffer, evaluate_candidates,
};
pub use crate::layer1::utility_types::AssignmentType;
use crate::layer1::utility_types::manhattan_distance;
use crate::layer1::utility_types::need_response_curve;
use crate::layer1::utility_types::{ActionType, UtilityWeights, calculate_context_score};
use bevy_ecs::prelude::*;

/// Component tracking what a pop is assigned to.
#[derive(Component, Debug)]
pub struct AssignedTo {
    /// The entity the pop is assigned to.
    pub entity: Entity,
    /// The type of assignment.
    pub assignment_type: AssignmentType,
}

/// Generic evaluator for simple actions (work, repair, etc.)
#[must_use]
pub fn evaluate_simple_action(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    candidates: &[ScorableCandidate],
    base_utility: f32,
) -> Option<(f32, Entity)> {
    evaluate_candidates(pop_pos, weights, candidates, base_utility)
}

/// Evaluates if a pop should fetch clothing.
///
/// # The "Shiver Factor"
///
/// Pops don't just upgrade clothes for fashion; they do it to survive.
/// This evaluator checks:
///
/// 1.  **Nakedness:** If `insulation == 0.0`, urgency is high (0.95).
/// 2.  **Hypothermia Risk:** If the local temperature is below what the current
///     clothes can handle (`insulation * -30 + 10`), urgency becomes critical (0.99),
///     prompting an immediate upgrade even if they are already dressed.
#[must_use]
pub fn evaluate_fetch_clothing(
    pop_pos: GridPosition,
    current_insulation: f32,
    resources: &ColonyResources,
    stockpiles: &[ScorableCandidate],
    temperature_grid: Option<&crate::layer1::temperature::TemperatureGrid>,
) -> Option<(f32, Entity)> {
    let mut score = 0.0;

    if current_insulation == 0.0 {
        score = 0.95; // Need clothes!
    } else if let Some(grid) = temperature_grid {
        // Check if freezing despite clothes
        #[allow(clippy::cast_sign_loss)]
        let temp = grid.get(pop_pos.x as usize, pop_pos.y as usize);
        let safe_temp = current_insulation.mul_add(-30.0, 10.0);
        if temp < safe_temp {
            // If insulation is already high (e.g. >= 2.0), fetching won't help unless we have super-parka.
            if current_insulation < 2.0 {
                score = 0.99; // Upgrade needed immediately!
            }
        }
    }

    if score == 0.0 {
        return None;
    }

    // If no clothing available in colony, can't fetch
    if resources.clothing < 1.0 {
        return None;
    }

    // Use evaluate_candidates with default weights (clothing is basic need, traits matter less)
    let weights = UtilityWeights::default();
    evaluate_candidates(pop_pos, &weights, stockpiles, score)
}

/// Evaluates if a pop should fetch a tool.
#[must_use]
pub fn evaluate_fetch_tool(
    pop_pos: GridPosition,
    equipment: &Equipment,
    resources: &ColonyResources,
    stockpiles: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // If already has a tool, no need to fetch
    if equipment.tool.is_some() {
        return None;
    }

    // If no tools available in colony, can't fetch
    if resources.tools < 1.0 {
        return None;
    }

    let weights = UtilityWeights::default();
    evaluate_candidates(pop_pos, &weights, stockpiles, 0.9)
}

/// Evaluates the utility of performing scientific research.
#[must_use]
pub fn evaluate_research(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    libraries: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // If knowledge is full, no utility
    if resources.knowledge >= resources.max_knowledge {
        return None;
    }

    evaluate_candidates(pop_pos, weights, libraries, 0.4)
}

/// Evaluates the utility of hauling loose items.
///
/// # Logistics Priority
///
/// Not all items are created equal. The AI prioritizes destinations:
///
/// 1.  **Gene Banks (Critical):** If carrying a `GeneticSample`, finding a Gene Bank
///     is Priority #1 (0.95 utility). We don't want rare DNA rotting in a pocket.
/// 2.  **Stockpiles (Standard):** If carrying anything else, find a Stockpile (0.90).
/// 3.  **Pickup (Idle):** If empty-handed, find the nearest loose item (0.60 base).
///     This is a lower priority than "real work" (Mining/Building).
#[must_use]
pub fn evaluate_haul(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    items: &[ScorableCandidate],
    item_entities: &[ScorableCandidate],
    stockpiles: &[ScorableCandidate],
    gene_banks: &[ScorableCandidate],
    resources: &ColonyResources,
    carrying: Option<crate::layer1::resources::Carrying>,
    carrying_item: Option<Entity>,
    carrying_item_type: Option<crate::layer1::items::ItemType>,
) -> Option<(f32, Entity)> {
    // 1. If carrying GeneticSample, target GeneBank
    if carrying_item.is_some()
        && matches!(
            carrying_item_type,
            Some(crate::layer1::items::ItemType::GeneticSample)
        )
    {
        if !gene_banks.is_empty() {
            return evaluate_candidates(pop_pos, weights, gene_banks, 0.95);
        }
        // If no gene banks, might fall through or fail.
        // For now, let it fall through to stockpiles if any (though unlikely to accept it if filtering implemented)
    }

    // 2. Check if any stockpile exists (Standard Hauling)
    if stockpiles.is_empty() {
        // If we are carrying something but no stockpile exists, and it wasn't a genetic sample for a gene bank,
        // we are stuck.
        return None;
    }

    // 3. If already carrying, go to stockpile (high priority)
    if carrying.is_some() || carrying_item.is_some() {
        return evaluate_candidates(pop_pos, weights, stockpiles, 0.9);
    }

    // 3. Find best item to pick up
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.6;

    // Helper closure to update best
    let mut consider = |candidate: &ScorableCandidate| {
        let context = calculate_context_score(
            pop_pos,
            Some(candidate.pos),
            candidate.capacity,
            candidate.usage,
            weights,
        );
        let utility = base_utility * context;
        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, candidate.entity));
        }
    };

    // Check Resource Items (with capacity check)
    for item in items {
        if let Some(res_type) = item.resource_type {
            let has_room = match res_type {
                crate::layer1::resources::ResourceType::Food => resources.food < resources.max_food,
                crate::layer1::resources::ResourceType::Wood => resources.wood < resources.max_wood,
                crate::layer1::resources::ResourceType::Stone => {
                    resources.stone < resources.max_stone
                }
                crate::layer1::resources::ResourceType::Ore => resources.ore < resources.max_ore,
                crate::layer1::resources::ResourceType::Metal => {
                    resources.metal < resources.max_metal
                }
                crate::layer1::resources::ResourceType::Planks => {
                    resources.planks < resources.max_planks
                }
                crate::layer1::resources::ResourceType::Blocks => {
                    resources.blocks < resources.max_blocks
                }
                crate::layer1::resources::ResourceType::Waste => {
                    resources.waste < resources.max_waste
                }
                crate::layer1::resources::ResourceType::Rations => {
                    resources.rations < resources.max_rations
                }
                crate::layer1::resources::ResourceType::Fuel => resources.fuel < resources.max_fuel,
                crate::layer1::resources::ResourceType::Alcohol => {
                    resources.alcohol < resources.max_alcohol
                }
                crate::layer1::resources::ResourceType::Scrap => {
                    resources.scrap < resources.max_scrap
                }
                crate::layer1::resources::ResourceType::Tools => {
                    resources.tools < resources.max_tools
                }
                crate::layer1::resources::ResourceType::BuildingPermit => {
                    resources.building_permits < resources.max_building_permits
                }
            };

            if !has_room {
                continue;
            }
        }
        consider(item);
    }

    // Check Generic Item Entities
    for item in item_entities {
        consider(item);
    }

    best
}

/// Evaluates the desire to listen to "The Hum".
#[must_use]
pub fn evaluate_listen_to_hum(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> (ActionType, f32, Option<Entity>) {
    // 1. Check Trait (Early Exit)
    if !data
        .traits
        .as_ref()
        .map_or(false, |t| t.0.contains(&Trait::Sensitive))
    {
        return (ActionType::ListenToTheHum, 0.0, None);
    }

    let mut best_score = 0.0;
    let mut best_target = None;

    for candidate in &buffer.hum_sources {
        let intensity = candidate.score_bonus;
        let desire = intensity * (1.0 + data.stress);

        let context_score = calculate_context_score(
            data.pos,
            Some(candidate.pos),
            candidate.capacity,
            candidate.usage,
            &data.weights,
        );

        let total_score = desire * context_score;

        if total_score > best_score {
            best_score = total_score;
            best_target = Some(candidate.entity);
        }
    }

    (ActionType::ListenToTheHum, best_score, best_target)
}

/// Evaluates actions for a drafted pop (combat).
pub fn evaluate_drafted_behavior(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> Option<(ActionType, f32, Option<Entity>)> {
    data.drafted?;

    let mut best_action = ActionType::Idle;
    let mut best_utility = 0.9; // Just stand there ready
    let mut best_target = None;

    // Helper to evaluate fight
    let evaluate_fight = || -> Option<(f32, Entity)> {
        // Find nearest enemy
        let mut best_fight_target = None;
        let mut min_dist = f32::MAX;

        for candidate in &buffer.enemies {
            #[allow(clippy::cast_precision_loss)]
            let dist = data.pos.distance_chebyshev(candidate.pos) as f32;
            if dist < min_dist {
                min_dist = dist;
                best_fight_target = Some(candidate.entity);
            }
        }

        if let Some(target) = best_fight_target {
            // High score for combat when drafted
            Some((0.95 - (min_dist * 0.01).min(0.5), target))
        } else {
            None
        }
    };

    if let Some((utility, target)) = evaluate_fight() {
        best_action = ActionType::Fight;
        best_utility = utility;
        best_target = Some(target);
    }

    Some((best_action, best_utility, best_target))
}

/// Evaluates actions for a pop undergoing a mental break.
pub fn evaluate_mental_break(
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

/// Evaluates the utility of using a shower.
#[must_use]
pub fn evaluate_shower(
    pop_pos: GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    candidates: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // Check if we can afford a shower
    if resources.water < SHOWER_WATER_COST {
        return None;
    }

    let urgency = need_response_curve(needs.hygiene);

    // If hygiene is high, urgency is low.
    // If urgency is very low, don't bother scanning.
    if urgency < 0.1 {
        return None;
    }

    evaluate_candidates(pop_pos, weights, candidates, urgency)
}

/// Evaluates the utility of cleaning clutter.
#[must_use]
pub fn evaluate_clean(
    pop_pos: GridPosition,
    weights: &UtilityWeights,
    candidates: &[ScorableCandidate],
    is_janitor: bool,
) -> Option<(f32, Entity)> {
    if candidates.is_empty() {
        return None;
    }

    // Janitors prioritize cleaning. Idle pops do it as community service if bad enough.
    // Base utility:
    // Janitor: 0.6 (Good priority)
    // Idle: 0.1 (Low priority, requires critical clutter)

    let base_utility = if is_janitor { 0.6 } else { 0.1 };

    // Find best target.
    // Score bonus in candidate is clutter amount / 100.0.
    // So 80.0 clutter = 0.8 bonus.
    // Janitor: 0.6 + 0.8 = 1.4 (Very high)
    // Idle: 0.1 + 0.8 = 0.9 (High enough to beat Idle 0.05)
    //
    // If clutter is 40.0:
    // Janitor: 0.6 + 0.4 = 1.0
    // Idle: 0.1 + 0.4 = 0.5 (Might beat explore/idle)

    // Filter for non-janitors: Only accept if clutter > 0.8 (Critical)
    // We can filter candidates or just rely on scoring.
    // Let's rely on scoring but enforce a threshold for non-janitors.

    let mut best: Option<(f32, Entity)> = None;

    for candidate in candidates {
        // Skip if not janitor and clutter not critical
        if !is_janitor && candidate.score_bonus < 0.8 {
            continue;
        }

        let context = calculate_context_score(
            pop_pos,
            Some(candidate.pos),
            candidate.capacity,
            candidate.usage,
            weights,
        );

        let utility = (base_utility + candidate.score_bonus) * context;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, candidate.entity));
        }
    }
    best
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
