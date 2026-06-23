# 1125 - Improvised Tools

## 1. Overview

**Layer:** 1
**Fantasy:** The desperation of using a rock when the hammer breaks.
**Mechanic:** When a specific tool is missing, Pops will grab raw materials (Stone, Wood, Scrap) to use as a temporary, low-durability tool. Consumes the material per use. 50% slower work speed.

This spec implements the `ImprovisedTools` component, and necessary data structure for evaluating task fallback logic to find raw materials.

## 2. Dependencies

- Layer 1 `Pop`, `Equipment`, `Tool`, `Inventory` and task structures.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_improvised_tools_fallback() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Task { tool_required: ToolType::Hammer },
            Inventory { resources: vec![Resource::Stone] },
        )).id();

        app.add_systems(Update, evaluate_tool_fallback_system);

        // Act
        app.update();

        // Assert
        let task = app.world().get::<Task>(pop_entity).unwrap();
        assert_eq!(task.using_improvised_tool, true);
        assert_eq!(task.efficiency, 0.5);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    Hammer,
    Pickaxe,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Stone,
    Wood,
    Scrap,
}

#[derive(Component)]
pub struct Inventory {
    pub resources: Vec<Resource>,
}

#[derive(Component)]
pub struct Task {
    pub tool_required: ToolType,
    pub using_improvised_tool: bool,
    pub efficiency: f32,
}

pub fn evaluate_tool_fallback_system(
    mut query: Query<(&mut Task, &Inventory)>,
) {
    for (mut task, inventory) in query.iter_mut() {
        // Simple logic for MVP, if missing tool and have stone
        if !task.using_improvised_tool && inventory.resources.contains(&Resource::Stone) {
            task.using_improvised_tool = true;
            task.efficiency = 0.5;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: Expand inventory tracking beyond a simple Vec to map or dictionary for fast lookup.
- **Design**: Generalize fallback mapping (e.g. Stone acts as Hammer, Scrap acts as Knife).
- **Integration**: Tie improvised tool usage to morale penalty or increased injury risk.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops without proper tools fallback to using raw materials.

## 7. Technical Guidance

- Implement inside the primary job assignment or equipment fetching system.

## 8. Questions

*Builder: add questions here if spec is unclear.*


## Questions
- Architectural Contradictions: `Task`, `ToolType`, and `Resource` enums defined in RED phase do not align with existing components. There is a `CurrentTask` component, and resources/tools are typically defined via `ItemType`. Therefore, the RED phase tests and GREEN phase logic are architecturally incompatible. I will pick another task from the backlog.
*Architect:* Yes, please adapt the RED phase tests and GREEN phase logic to use the existing `CurrentTask` and `ItemType` architecture rather than the non-existent enums. The intent of the spec is to implement tools improvised from scrap; the implementation details should map to the current codebase reality.
