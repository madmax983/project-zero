# 058: Personal Tools

## Overview

Transition from a global "Tools" resource pool (abstract counter) to **Personal Tools** (concrete items carried by Pops).

Currently, `ColonyResources.tools` acts as a global pool. When a Pop works, they check this pool.
This spec introduces:
1.  **Equipment Component**: Pops have slots for items (currently just `Tool`).
2.  **Tool Item**: An entity representing a physical tool with durability.
3.  **Fetch Tool Action**: Pops without a tool will go to a Stockpile to pick one up.
4.  **Durability**: Working reduces the durability of the specific tool equipped.
5.  **Breakage**: When durability hits 0, the tool entity is destroyed, and the Pop must fetch a new one.

This adds depth to logistics: tool shortages now cause Pops to stop working to fetch replacements, creating traffic and bottlenecks.

## Dependencies

- `030` — Tool Economy (Global resource foundation)
- `016` — Utility AI (Action evaluation)
- `056` — Designated Zones (Stockpile locations)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/personal_tools_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Equipment};
    use crate::layer1::items::{Item, Tool, ToolType};
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::execution::work_execution_system;
    use crate::layer1::resources::{ColonyResources, MiningProgress};
    use crate::layer1::map::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};

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
        let tool = world.spawn((
            Item,
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            }
        )).id();

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

        // Spawn Tool
        let tool = world.spawn(Tool {
            tool_type: ToolType::Pickaxe,
            durability: 10.0,
            max_durability: 100.0,
        }).id();

        // Spawn Designation (Mine)
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            MiningProgress::default(),
        )).id();

        // Spawn Pop with Equipment
        world.spawn((
            Pop,
            Equipment { tool: Some(tool) },
            GridPosition { x: 5, y: 5 },
            // Add work components
            crate::layer1::execution::MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            crate::layer1::execution::AtTarget,
            PopAction { current: ActionType::Work, ..Default::default() },
        ));

        // Run execution system
        work_execution_system(&mut world);

        // Check durability reduced
        let t = world.get::<Tool>(tool).unwrap();
        assert!(t.durability < 10.0);
    }

    #[test]
    fn test_tool_breakage_removes_item() {
        let mut world = World::new();
        // Setup
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::edicts::ColonyPolicies::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        let tool = world.spawn(Tool {
            tool_type: ToolType::Pickaxe,
            durability: 0.01, // Almost broken
            max_durability: 100.0,
        }).id();

        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            MiningProgress::default(),
        )).id();

        let pop = world.spawn((
            Pop,
            Equipment { tool: Some(tool) },
            GridPosition { x: 5, y: 5 },
            crate::layer1::execution::MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            crate::layer1::execution::AtTarget,
            PopAction { current: ActionType::Work, ..Default::default() },
        )).id();

        // Force durability to 0 or simulate enough work to break it
        // For test, we rely on system logic.
        // Assuming system subtracts > 0.01

        work_execution_system(&mut world);

        // Tool entity should be despawned
        assert!(world.get_entity(tool).is_none());

        // Pop equipment should be None
        let eq = world.get::<Equipment>(pop).unwrap();
        assert!(eq.tool.is_none());
    }

    #[test]
    fn test_fetch_tool_action_logic() {
        // Evaluate should return high score if no tool and tools available
        // This requires integrating with evaluate_actions_system or a specific evaluate_fetch_tool function
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components

Create `src/layer1/items.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct Item;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Hammer, // For building
}

#[derive(Component, Debug, Clone)]
pub struct Tool {
    pub tool_type: ToolType,
    pub durability: f32,
    pub max_durability: f32,
}

#[derive(Component, Debug, Default)]
pub struct Equipment {
    pub tool: Option<Entity>,
}
```

### 2. Update `ActionType`

In `src/layer1/utility_ai/types.rs`:
- Add `FetchTool` variant.
- Update `ActionType::COUNT` to **12**.
- Update `as_index`.

**CRITICAL:** You must also update:
- `src/gpu/buffers.rs`: Update `GpuPopInput` array sizes.
- `src/gpu/shaders/evaluate.wgsl`: Add the new action index to any arrays or logic.

### 3. Implement `FetchTool` Logic

Create `src/layer1/actions/fetch_tool.rs`:

```rust
pub fn evaluate_fetch_tool(
    pop_entity: Entity,
    equipment: &Equipment,
    resources: &ColonyResources,
    stockpiles: Query<(Entity, &GridPosition), With<Stockpile>>,
    // ...
) -> Option<(f32, Entity)> {
    if equipment.tool.is_some() { return None; }
    if resources.tools < 1.0 { return None; }

    // Find nearest stockpile
    // Return high score (e.g. 0.9 if toolless)
}
```

And handle arrival in `execution.rs`:

```rust
// In arrival_handler_system match action:
ActionType::FetchTool => {
    // Check if resources.tools >= 1.0
    // Decrement resources.tools
    // Spawn Tool entity
    // Insert into Equipment.tool
}
```

### 4. Update `work_execution_system`

Modify `src/layer1/execution.rs`:
- Instead of checking `resources.tools`, check `query_pops.get(pop_entity).equipment.tool`.
- Calculate efficiency based on tool presence (1.0 vs 0.5).
- Decrease durability of the tool entity.
- If `durability <= 0.0`:
    - Despawn tool entity.
    - Set `equipment.tool = None`.
    - Log "Tool broken".

## REFACTOR Phase: Quality & Design

- **Visuals**: Pops holding tools should ideally show them (later).
- **Tool Types**: Currently generic. Ideally `Mine` needs `Pickaxe`, `Chop` needs `Axe`. For MVP, any tool works for any job, or generic "Tool".
- **Refining**: `Smithy` currently produces `ColonyResources.tools`. This stays the same; the "Resource" is just a counter of uninstantiated tools.
- **Cleanup**: Remove the old "random break chance" logic from `ColonyResources` based breakage. Breakage is now deterministic (durability).

## Acceptance Criteria

- [ ] Pops have `Equipment` component.
- [ ] Working with a tool reduces its specific durability.
- [ ] Tool entity despawns when broken.
- [ ] Pops automatically seek `FetchTool` action when they have no tool and `ColonyResources.tools > 0`.
- [ ] `ActionType` updated correctly in Rust and GPU code.
- [ ] Tests pass.

## Technical Guidance

- **GPU Shader**: If you forget to update the shader array size, the game might crash or render incorrectly.
- **Resource Lock**: Be careful borrowing `ColonyResources` mutably in `execution.rs` if you also need it immutably for other checks.
- **Borrow Checker**: You'll likely need to query `Equipment` and `Tool` separately or use `get_component` to avoid borrowing the whole world.
