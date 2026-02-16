use crate::layer1::items::{Equipment, Item, Tool, ToolType};
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::PositionProxy;
use crate::layer1::utility_types::manhattan_distance;
use bevy_ecs::prelude::*;

/// Evaluates if a pop should fetch a tool.
#[must_use]
pub(crate) fn evaluate_fetch_tool(
    pop_pos: &GridPosition,
    equipment: &Equipment,
    resources: &ColonyResources,
    stockpiles: &[PositionProxy],
) -> Option<(f32, Entity)> {
    // If already has a tool, no need to fetch
    if equipment.tool.is_some() {
        return None;
    }

    // If no tools available in colony, can't fetch
    if resources.tools < 1.0 {
        return None;
    }

    // Find nearest stockpile
    // Note: Stockpiles don't explicitly "contain" tools in the current model,
    // they just represent storage. We assume tools are at stockpiles.
    // If no stockpiles exist, we can't fetch.

    let mut best_target = None;
    let mut min_dist = i32::MAX;

    for stockpile in stockpiles {
        let dist = manhattan_distance(pop_pos, &stockpile.pos);
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(stockpile.entity);
        }
    }

    best_target.map(|target| {
        // High urgency if no tool, as tools are critical for work efficiency.
        // But maybe not as critical as hunger/rest?
        // Let's say 0.9.
        // Distance penalty?
        // Similar to other actions: utility = base * distance_factor.
        // Let's use a simple distance factor for now.
        // 1.0 / (dist * 0.1 + 1.0)

        #[allow(clippy::cast_precision_loss)]
        let distance_factor = 1.0 / (min_dist as f32).mul_add(0.1, 1.0);
        let utility = 0.9 * distance_factor;

        (utility, target)
    })
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
                Item,
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
