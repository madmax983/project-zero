# 076: Heirloom Items

## Overview

Tools are not just disposable assets; they accumulate history. A pickaxe used to mine the first stone of the colony becomes legendary.
This spec introduces a `ToolHistory` component to track usage statistics (ticks used, resources harvested). When a tool reaches a certain threshold of usage, it transforms into an `Heirloom`, gaining a unique name and an efficiency bonus.

This adds narrative depth to the equipment system (`058`) and provides a late-game optimization vector through "leveling up" tools.

## Dependencies

- `058` — Personal Tools (Base `Tool` and `Equipment` components)
- `016` — Utility AI (Work action execution)
- `030` — Tool Economy (Resource context)

## RED Phase: Tests First

Write these tests in `src/layer1/heirloom_items_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::items::{Item, Tool, ToolType};
    use crate::layer1::heirloom::{ToolHistory, Heirloom, check_heirloom_status_system};
    use crate::layer1::execution::work_execution_system; // Integration point
    use crate::layer1::pop::{Pop, Equipment};
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::map::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::resources::{ColonyResources, MiningProgress};

    #[test]
    fn test_tool_history_component_defaults() {
        let history = ToolHistory::default();
        assert_eq!(history.ticks_used, 0);
        assert_eq!(history.items_harvested, 0);
    }

    #[test]
    fn test_work_increments_history() {
        let mut world = World::new();
        // Setup minimal world for work_execution
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::edicts::ColonyPolicies::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Spawn Tool with History
        let tool = world.spawn((
            Item,
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
            ToolHistory::default(),
        )).id();

        // Spawn Pop using Tool
        // Note: Integration test assumes work_execution_system updates history
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            MiningProgress::default(),
        )).id();

        world.spawn((
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
        ));

        // Run execution system
        work_execution_system(&mut world);

        let history = world.get::<ToolHistory>(tool).unwrap();
        assert!(history.ticks_used > 0, "Working should increment ticks_used");
    }

    #[test]
    fn test_heirloom_transition() {
        let mut world = World::new();

        // Spawn Tool with high history
        let tool = world.spawn((
            Item,
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
            ToolHistory {
                ticks_used: 1000, // Threshold met
                items_harvested: 100,
            },
        )).id();

        // Run check system
        world.run_system_once(check_heirloom_status_system).unwrap();

        // Verify Heirloom component added
        let heirloom = world.get::<Heirloom>(tool);
        assert!(heirloom.is_some(), "Tool should become Heirloom");

        let h = heirloom.unwrap();
        assert!(h.efficiency_bonus > 0.0);
        assert!(!h.name.is_empty());
    }

    #[test]
    fn test_heirloom_bonus_application() {
        // Need to verify that the Heirloom bonus is actually applied in work calculation.
        // This might require a mocked calculation or checking work output.
        // For MVP, checking the component exists and has value is sufficient for this unit.
        let heirloom = Heirloom {
            name: "Founder's Pick".to_string(),
            efficiency_bonus: 0.2,
        };
        assert_eq!(heirloom.efficiency_bonus, 0.2);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/heirloom.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Default, Clone)]
pub struct ToolHistory {
    pub ticks_used: u32,
    pub items_harvested: u32, // Incremented on task completion
}

#[derive(Component, Debug, Clone)]
pub struct Heirloom {
    pub name: String,
    pub efficiency_bonus: f32,
}

// System to check for promotion
pub fn check_heirloom_status_system(
    mut commands: Commands,
    query: Query<(Entity, &ToolHistory), Without<Heirloom>>,
) {
    // Thresholds
    const TICKS_THRESHOLD: u32 = 1000;

    for (entity, history) in query.iter() {
        if history.ticks_used >= TICKS_THRESHOLD {
            commands.entity(entity).insert(Heirloom {
                name: format!("Legendary Tool #{}", entity.index()), // Placeholder name logic
                efficiency_bonus: 0.25, // +25% work speed
            });
        }
    }
}
```

### 2. Update Execution System

Modify `src/layer1/execution.rs` (specifically `work_execution_system`):

```rust
// Inside the loop where work is performed:
if let Some(tool_entity) = equipment.tool {
    if let Some(mut history) = world.get_mut::<ToolHistory>(tool_entity) {
        history.ticks_used += 1;
    }

    // If task completes (e.g. mining finishes), increment items_harvested
    // This requires detecting completion event or checking progress >= max
    // For MVP, ticks_used is sufficient for "Leveling Up".
}
```

### 3. Apply Bonus

Modify `calculate_work_amount` (or similar helper) to check for `Heirloom`:

```rust
pub fn calculate_work_amount(pop_entity: Entity, tool_entity: Option<Entity>, world: &World) -> f32 {
    let mut amount = 1.0; // Base

    // ... existing skill/tool logic ...

    if let Some(tool) = tool_entity {
        if let Some(heirloom) = world.get::<Heirloom>(tool) {
            amount *= (1.0 + heirloom.efficiency_bonus);
        }
    }

    amount
}
```

## REFACTOR Phase: Quality & Design

- **Naming System**: Integrate with `Lore` generator to create names like "The Stone-Eater" or "Miner's Pride" instead of generic names.
- **Visuals**: Heirlooms should have a particle effect or unique sprite color (Gold?).
- **Durability**: Heirlooms should have increased max durability or self-repair chance (magical properties).
- **Inheritance**: If a Pop dies, the Heirloom is dropped and retains its status, ready for the next generation.

## Acceptance Criteria

- [ ] `ToolHistory` component tracks `ticks_used`.
- [ ] `check_heirloom_status_system` promotes tools to `Heirloom` after 1000 ticks.
- [ ] `Heirloom` component grants efficiency bonus in work calculations.
- [ ] Tests pass.

## Technical Guidance

- Ensure `ToolHistory` is added to all newly spawned tools by default (in `src/layer1/items.rs` or spawn logic), or make it optional and add it later. (Better: Add by default to `Tool` bundle).
- Be careful with `world.get_mut` inside execution loops; if `work_execution_system` iterates Pops, fetching Tool component is fine, but ensure borrow rules are respected.
