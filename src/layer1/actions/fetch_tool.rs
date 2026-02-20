use crate::layer1::items::{Equipment, Item, Tool, ToolType};
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates if a pop should fetch a tool.
#[must_use]
pub(crate) fn evaluate_fetch_tool(
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

    // Use evaluate_candidates with default weights (implicitly handled by calculate_context_score inside)
    // We construct temporary weights if needed, or assume caller passes weights.
    // The original code used manhattan distance directly. evaluate_candidates uses weights.
    // Let's assume we want standard behavior.
    // But this function signature in the old code didn't take weights!
    // I need to update the signature to take weights.
    // utility_ai.rs passes it? No, it didn't!
    // Let's check utility_ai.rs again.

    // In utility_ai.rs:
    // evaluate_fetch_tool(pop_pos, &equipment, context.resources, &buffer.stockpiles)

    // So I need to update utility_ai.rs to pass weights to evaluate_fetch_tool?
    // Or I can use Default::default() weights here.

    let weights = UtilityWeights::default();
    evaluate_candidates(pop_pos, &weights, stockpiles, 0.9)
}

/// Executes the fetch tool action.
pub fn handle_fetch_tool(
    commands: &mut Commands,
    resources: &mut ColonyResources,
    pop_entity: Entity,
    equipment_opt: &mut Option<Mut<Equipment>>,
) {
    if resources.tools >= 1.0 {
        resources.tools -= 1.0;

        // Use a generic history default
        let tool_history = crate::layer1::heirloom::ToolHistory::default();

        let tool_entity = commands
            .spawn((
                Item::default(),
                Tool {
                    tool_type: ToolType::Pickaxe, // Generic for now
                    durability: 100.0,
                    max_durability: 100.0,
                },
                tool_history,
            ))
            .id();

        if let Some(eq) = equipment_opt {
            eq.tool = Some(tool_entity);
        } else {
            commands.entity(pop_entity).insert(Equipment {
                tool: Some(tool_entity),
                ..Default::default()
            });
        }
    }
}
