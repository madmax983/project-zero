use bevy_ecs::prelude::*;

/// Marker component for an item entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct Item;

/// Type of tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    /// Used for mining.
    Pickaxe,
    /// Used for forestry.
    Axe,
    /// Used for building.
    Hammer, // For building
}

/// Component representing a physical tool with durability.
#[derive(Component, Debug, Clone)]
pub struct Tool {
    /// The type of tool.
    pub tool_type: ToolType,
    /// Current durability remaining.
    pub durability: f32,
    /// Maximum durability.
    pub max_durability: f32,
}

/// Component for equipment slots on a Pop.
#[derive(Component, Debug, Default, Clone)]
pub struct Equipment {
    /// The entity ID of the equipped tool.
    pub tool: Option<Entity>,
    /// The entity ID of the equipped weapon.
    pub weapon: Option<Entity>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::execution::work_execution_system;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, MiningProgress};
    use crate::layer1::utility_ai::{ActionType, PopAction};

    #[test]
    fn test_equipment_component_exists() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Equipment::default())).id();
        let eq = world.get::<Equipment>(entity).unwrap();
        assert!(eq.tool.is_none());
    }

    #[test]
    fn test_tool_item_component() {
        let mut world = World::new();
        let tool = world
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        let t = world.get::<Tool>(tool).unwrap();
        assert_eq!(t.durability, 100.0);
    }

    #[test]
    fn test_work_reduces_durability() {
        let mut world = World::new();
        // Setup World Resources
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::edicts::ColonyPolicies::default());
        world.insert_resource(crate::shared::log::MessageLog::default());
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });

        // Spawn Tool
        let tool = world
            .spawn(Tool {
                tool_type: ToolType::Pickaxe,
                durability: 10.0,
                max_durability: 100.0,
            })
            .id();

        // Spawn Designation (Mine)
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
                MiningProgress::default(),
            ))
            .id();

        // Spawn Pop with Equipment
        world.spawn((
            Pop,
            Equipment { tool: Some(tool), ..Default::default() },
            GridPosition { x: 5, y: 5 },
            // Add work components
            crate::layer1::execution::MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            crate::layer1::execution::AtTarget,
            PopAction {
                current: ActionType::Work,
                ..Default::default()
            },
        ));

        // Run execution system
        work_execution_system(&mut world);

        // Check durability reduced
        let t = world.get::<Tool>(tool).unwrap();
        assert!(
            t.durability < 10.0,
            "Durability should decrease (was {})",
            t.durability
        );
    }

    #[test]
    fn test_tool_breakage_removes_item() {
        let mut world = World::new();
        // Setup
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::edicts::ColonyPolicies::default());
        world.insert_resource(crate::shared::log::MessageLog::default());
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });

        // Spawn Tool with VERY low durability
        let tool = world
            .spawn(Tool {
                tool_type: ToolType::Pickaxe,
                durability: 0.00001, // Almost broken, effectively 0 after any work
                max_durability: 100.0,
            })
            .id();

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
                MiningProgress::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                Equipment { tool: Some(tool), ..Default::default() },
                GridPosition { x: 5, y: 5 },
                crate::layer1::execution::MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                crate::layer1::execution::AtTarget,
                PopAction {
                    current: ActionType::Work,
                    ..Default::default()
                },
            ))
            .id();

        // Force durability to 0 or simulate enough work to break it
        work_execution_system(&mut world);

        // Tool entity should be despawned
        assert!(
            world.get_entity(tool).is_err(),
            "Tool entity should be despawned"
        );

        // Pop equipment should be None
        let eq = world.get::<Equipment>(pop).unwrap();
        assert!(eq.tool.is_none(), "Pop equipment should be cleared");
    }
}
