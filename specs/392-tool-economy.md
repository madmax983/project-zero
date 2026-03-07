# 392 - Tool Economy

## 1. Overview
The Tool Economy introduces durability and necessity to Pops' equipment. Jobs like Mining or Farming require specific tools (Pickaxe, Hoe). Tools degrade with use. When a tool breaks, work speed slows massively until it is replaced. This creates an economic dependency on the Smithy/Crafting system to constantly replenish the tool supply.

## 2. Dependencies
- `009-job-system.md` (Work assignment and tasks)
- `021-utility-ai-work.md` (Utility AI for work actions)
- `058-personal-tools.md` (Existing tool framework to build upon)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::job::Job;
    use crate::layer1::item::{ToolType, ToolDurability};
    use crate::layer1::pop::Equipment;

    #[test]
    fn test_tool_durability_drain() {
        let mut app = App::new();
        app.add_systems(Update, apply_tool_wear);

        let mut equipment = Equipment::default();
        equipment.set_tool(ToolType::Pickaxe, ToolDurability { current: 100, max: 100 });

        let pop = app.world_mut().spawn((
            equipment,
            Job { task: "Mine", ..default() },
            WorkAction { progress: 0.0, active: true },
        )).id();

        // Simulate work cycle
        app.update();

        // Tool should have lost durability
        let eq = app.world().get::<Equipment>(pop).unwrap();
        let dur = eq.get_tool_durability(ToolType::Pickaxe).unwrap();
        assert!(dur.current < 100, "Durability should decrease on active work");
    }

    #[test]
    fn test_tool_breakage_slows_work() {
        let mut app = App::new();
        app.add_systems(Update, calculate_work_speed);

        // Pop with a broken tool
        let mut equipment = Equipment::default();
        equipment.set_tool(ToolType::Pickaxe, ToolDurability { current: 0, max: 100 });

        let pop = app.world_mut().spawn((
            equipment,
            Job { task: "Mine", ..default() },
            WorkSpeed(1.0),
        )).id();

        app.update();

        // Speed should be massively reduced
        let speed = app.world().get::<WorkSpeed>(pop).unwrap();
        assert!(speed.0 < 1.0, "Broken tool should apply severe work penalty");
        assert_eq!(speed.0, 0.1); // e.g., 90% penalty
    }

    #[test]
    fn test_utility_ai_seeks_replacement_tool() {
        let mut app = App::new();
        // Setup utility AI scoring for "Equip Tool" action
        app.add_systems(Update, score_equip_tool_action);

        // Broken tool should increase desire to equip a new one
        let mut equipment = Equipment::default();
        equipment.set_tool(ToolType::Pickaxe, ToolDurability { current: 0, max: 100 });

        let pop = app.world_mut().spawn((
            equipment,
            Job { task: "Mine", ..default() },
            UtilityScores::default(),
        )).id();

        // Create a stockpile with a new pickaxe
        app.world_mut().spawn((
            Stockpile,
            Inventory::with_item(Item::Tool(ToolType::Pickaxe)),
        ));

        app.update();

        let scores = app.world().get::<UtilityScores>(pop).unwrap();
        let equip_score = scores.get("EquipTool");
        assert!(equip_score > 0.8, "Utility AI should heavily prioritize replacing a broken required tool");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Hoe,
}

#[derive(Clone, Copy)]
pub struct ToolDurability {
    pub current: i32,
    pub max: i32,
}

#[derive(Component, Default)]
pub struct Equipment {
    pub tools: HashMap<ToolType, ToolDurability>,
}

impl Equipment {
    pub fn set_tool(&mut self, tool: ToolType, dur: ToolDurability) {
        self.tools.insert(tool, dur);
    }
    pub fn get_tool_durability(&self, tool: ToolType) -> Option<&ToolDurability> {
        self.tools.get(&tool)
    }
    pub fn degrade_tool(&mut self, tool: ToolType, amount: i32) {
        if let Some(dur) = self.tools.get_mut(&tool) {
            dur.current = (dur.current - amount).max(0);
        }
    }
}

#[derive(Component)]
pub struct Job {
    pub task: String,
    pub required_tool: Option<ToolType>,
}

#[derive(Component)]
pub struct WorkAction {
    pub active: bool,
    pub progress: f32,
}

#[derive(Component)]
pub struct WorkSpeed(pub f32);

pub fn apply_tool_wear(
    mut query: Query<(&mut Equipment, &Job, &WorkAction)>
) {
    for (mut eq, job, action) in query.iter_mut() {
        if action.active {
            if let Some(tool) = job.required_tool {
                eq.degrade_tool(tool, 1);
            }
        }
    }
}

pub fn calculate_work_speed(
    mut query: Query<(&Equipment, &Job, &mut WorkSpeed)>
) {
    for (eq, job, mut speed) in query.iter_mut() {
        if let Some(tool) = job.required_tool {
            if let Some(dur) = eq.get_tool_durability(tool) {
                if dur.current == 0 {
                    speed.0 = 0.1; // 90% penalty for broken tool
                } else {
                    speed.0 = 1.0; // Normal speed
                }
            } else {
                 speed.0 = 0.1; // Missing tool
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Tool Breakage Event**: Emit a `ToolBrokenEvent` when `current` hits 0. This can trigger a notification for the player or specific UI animations.
- **Crafting Integration**: The `WorkSpeed` penalty will naturally create the "Death Spiral" if the Smithy can't produce enough tools fast enough. Ensure the Smithy job has appropriate priority.
- **Repair Mechanics**: Allow tools to be repaired before they break fully, saving resources compared to building entirely new ones.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥ 85% for tool durability components
- [ ] Tool wear applies during active work.
- [ ] Broken tools apply a 90% work speed penalty.
- [ ] Utility AI scores replacing a broken tool very highly.

## 7. Technical Guidance
- `Equipment` should be integrated into `layer1/pop.rs` or `layer1/inventory.rs`.
- Ensure the `score_equip_tool_action` considers the distance to the nearest stockpile containing the required tool.

## 8. Questions
*Builder: add questions here if spec is unclear.*
