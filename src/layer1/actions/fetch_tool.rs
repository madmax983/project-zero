use crate::layer1::items::{Equipment, Item, Tool, ToolType};
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::utility_ai::manhattan_distance;
use bevy_ecs::prelude::*;

/// Evaluates if a pop should fetch a tool.
pub fn evaluate_fetch_tool<'a>(
    pop_pos: &GridPosition,
    equipment: &Equipment,
    resources: &ColonyResources,
    stockpiles: impl Iterator<Item = (Entity, &'a GridPosition, &'a Stockpile)>,
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

    for (entity, pos, _) in stockpiles {
        let dist = manhattan_distance(pop_pos, pos);
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(entity);
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

        let tool_entity = commands
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe, // Generic for now
                    durability: 100.0,
                    max_durability: 100.0,
                },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::items::{Equipment, Tool, ToolType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::stockpile::Stockpile;

    #[test]
    fn test_evaluate_fetch_tool_no_tools_resource() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let equipment = Equipment::default(); // No tool
        let resources = ColonyResources {
            tools: 0.0,
            ..Default::default()
        };
        let stockpiles = vec![];

        let result = evaluate_fetch_tool(
            &pop_pos,
            &equipment,
            &resources,
            stockpiles.into_iter().map(|(e, p, s)| (e, p, s)),
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_fetch_tool_already_has_tool() {
        let mut world = World::new();
        let tool = world
            .spawn(Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            })
            .id();

        let pop_pos = GridPosition { x: 0, y: 0 };
        let equipment = Equipment {
            tool: Some(tool),
            ..Default::default()
        };
        let resources = ColonyResources {
            tools: 10.0,
            ..Default::default()
        };
        let stockpiles = vec![];

        let result = evaluate_fetch_tool(
            &pop_pos,
            &equipment,
            &resources,
            stockpiles.into_iter().map(|(e, p, s)| (e, p, s)),
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_fetch_tool_success() {
        let mut world = World::new();
        let stockpile_entity = world
            .spawn((Stockpile::default(), GridPosition { x: 5, y: 0 }))
            .id();
        let stockpile_pos = GridPosition { x: 5, y: 0 };
        let stockpile_comp = Stockpile::default();

        let pop_pos = GridPosition { x: 0, y: 0 };
        let equipment = Equipment::default();
        let resources = ColonyResources {
            tools: 1.0,
            ..Default::default()
        };

        // Mock iterator
        let stockpiles = vec![(stockpile_entity, &stockpile_pos, &stockpile_comp)];

        let result = evaluate_fetch_tool(&pop_pos, &equipment, &resources, stockpiles.into_iter());

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, stockpile_entity);
        assert!(utility > 0.0);
        // Distance 5. Factor = 1.0 / (5*0.1 + 1.0) = 1.0 / 1.5 = 0.66.
        // Utility = 0.9 * 0.66 = 0.6.
        assert!((utility - 0.6).abs() < 0.1);
    }
}
