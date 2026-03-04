# 030: Tool Economy

## Overview

Implement a tool economy where **Tools** are a resource required for efficient work. Jobs (Mining, Chopping) consume tools over time. A **Smithy** building produces Tools from Metal and Wood. This adds a maintenance loop to the game economy.

## Dependencies

- `024` — Metal Industry (for Metal resource and Smelter context)
- `016` — Utility AI (for work execution context)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/tool_tests.rs (or inside resources.rs/refining.rs tests)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{ColonyResources, mine_rock, MiningProgress};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::refining::get_refining_recipe;
    use crate::layer1::execution::work_execution_system;

    // 1. ColonyResources should have tools
    #[test]
    fn test_colony_resources_tools_fields() {
        let resources = ColonyResources::default();
        assert_eq!(resources.tools, 0.0);
        assert_eq!(resources.max_tools, 50.0); // Default cap
    }

    // 2. BuildingType::Smithy should exist
    #[test]
    fn test_building_type_smithy() {
        let bt = BuildingType::Smithy;
        assert_eq!(bt.label(), "Smithy");
        assert_eq!(bt.char(), 'T'); // T for Tool

        // Check cost (example: 30 Wood, 10 Stone)
        let cost = bt.cost();
        assert_eq!(cost.wood, 30.0);
        assert_eq!(cost.stone, 10.0);
    }

    // 3. Refining Recipe for Smithy
    #[test]
    fn test_smithy_recipe() {
        let mut resources = ColonyResources::default();
        resources.metal = 1.0;
        resources.wood = 1.0;
        resources.tools = 0.0;

        let (can_refine, input, output) = get_refining_recipe(BuildingType::Smithy, &resources);

        assert!(can_refine);
        assert_eq!(input.metal, 1.0);
        assert_eq!(input.wood, 1.0);
        assert_eq!(output.tools, 1.0);
    }

    // 4. Work Execution consumes tools (Probabilistic test - mocked or statistical)
    // For deterministic testing, we might need to expose the RNG or efficiency factor.
    // Instead, let's test that efficiency is calculated correctly based on tools.

    // We'll add a helper to calculate work efficiency in execution.rs
    /*
    #[test]
    fn test_work_efficiency_with_tools() {
        let resources = ColonyResources { tools: 10.0, ..Default::default() };
        let efficiency = calculate_tool_efficiency(&resources);
        assert_eq!(efficiency, 1.0);
    }

    #[test]
    fn test_work_efficiency_without_tools() {
        let resources = ColonyResources { tools: 0.0, ..Default::default() };
        let efficiency = calculate_tool_efficiency(&resources);
        assert_eq!(efficiency, 0.5);
    }
    */
}
```

## GREEN Phase: Minimal Implementation

### 1. Update ColonyResources

```rust
// src/layer1/resources.rs
#[derive(Resource, Debug, Clone)]
pub struct ColonyResources {
    // ... existing ...
    pub tools: f32,
    pub max_tools: f32,
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            // ... existing ...
            tools: 0.0,
            max_tools: 50.0,
            // ...
        }
    }
}

// Add add_tools method
```

### 2. Update BuildingType

```rust
// src/layer1/building.rs
pub enum BuildingType {
    // ...
    Smithy,
}

// Update label(), char(), cost(), next(), required_tech()
// Smithy should require Tech::MetalWorking
```

### 3. Update Refining Logic

```rust
// src/layer1/refining.rs
fn get_refining_recipe(bt: BuildingType, res: &ColonyResources) -> (bool, ColonyResources, ColonyResources) {
    match bt {
        // ...
        BuildingType::Smithy => (
            res.metal >= 1.0 && res.wood >= 1.0 && res.tools < res.max_tools,
            ColonyResources { metal: 1.0, wood: 1.0, ..Default::default() },
            ColonyResources { tools: 1.0, ..Default::default() },
        ),
        // ...
    }
}
```

### 4. Update Work Execution (Tool Consumption)

```rust
// src/layer1/execution.rs

pub fn work_execution_system(world: &mut World) {
    // Check tools at the start of the system
    let (has_tools, mut tool_broken) = {
        let res = world.resource::<ColonyResources>();
        (res.tools >= 1.0, false)
    };

    let efficiency = if has_tools { 1.0 } else { 0.5 };
    let work_amount = WORK_PER_TICK * efficiency;

    // ... loop over workers ...
        // call mine_rock / chop_tree with work_amount

        // If has_tools, 1% chance to break a tool per work tick
        if has_tools && !tool_broken {
             let mut rng = rand::thread_rng();
             if rng.gen_bool(0.01) {
                 tool_broken = true;
             }
        }
    // ... end loop ...

    // Apply tool breakage
    if tool_broken {
        let mut res = world.resource_mut::<ColonyResources>();
        if res.tools >= 1.0 {
            res.tools -= 1.0;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Constants**: Extract `TOOL_BREAK_CHANCE` (0.01) and `NO_TOOL_PENALTY` (0.5).
- **Feedback**: Log a message when a tool breaks? (Maybe too spammy).
- **UI**: Ensure Tools show up in the resource bar (`src/ui/status.rs`).

## Acceptance Criteria

- [ ] `ColonyResources` has `tools`.
- [ ] `Smithy` building exists and produces tools from Metal + Wood.
- [ ] Mining/Forestry speed is halved if `tools == 0`.
- [ ] Tools randomly decrease while working (consumption).
- [ ] Tests pass.

## Technical Guidance

- Modifying `work_execution_system` requires careful handling of the `ColonyResources` borrow if you need to read it (for efficiency) and write it (for breakage).
- Suggestion: Read `has_tools` into a bool first. Calculate `tool_damage` accumulators during the loop. Apply updates to `ColonyResources` after the loop.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
