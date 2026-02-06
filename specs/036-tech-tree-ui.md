# 036: Tech Tree UI

## Overview

Adds a user interface to visualize the Tech Tree and unlock technologies using the **Knowledge** resource (from Spec 029).
This UI is accessed via the `[T]` key and renders as an overlay (similar to the Chronicle).
It allows the player to:
1. View available technologies.
2. See costs and unlock status.
3. Unlock affordable technologies.

## Dependencies

- `029` — Knowledge System (Resource, Tech Enum, TechState)
- `012` — Input Architecture (InputRouter, InputContext)
- `003` — UI Layout (Rendering)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/ui/tech_tests.rs (or inside src/ui/tech.rs)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::tech::{Tech, TechState};
    use crate::layer1::resources::ColonyResources;
    use crate::ui::tech::{TechUiState, get_tech_list};

    #[test]
    fn test_tech_ui_state_resource_defaults() {
        let state = TechUiState::default();
        assert!(!state.is_open);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_tech_ui_navigation() {
        let mut state = TechUiState::default();
        let tech_count = 3;

        // Down
        state.next(tech_count);
        assert_eq!(state.selected_index, 1);
        state.next(tech_count);
        assert_eq!(state.selected_index, 2);
        state.next(tech_count);
        assert_eq!(state.selected_index, 0); // Wrap

        // Up
        state.prev(tech_count);
        assert_eq!(state.selected_index, 2); // Wrap back
    }

    #[test]
    fn test_unlock_action_via_function() {
        let mut world = World::new();
        let mut tech_state = TechState::default();
        let mut res = ColonyResources::default();
        res.knowledge = 100.0;

        world.insert_resource(tech_state);
        world.insert_resource(res);
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Setup UI State selecting first tech
        let ui_state = TechUiState { is_open: true, selected_index: 0 };

        let techs = get_tech_list();
        let target_tech = techs[0];

        // Perform unlock logic (simulating input handler logic)
        crate::layer1::tech::unlock_tech(&mut world, target_tech);

        assert!(world.resource::<TechState>().is_unlocked(target_tech));
        assert!(world.resource::<ColonyResources>().knowledge < 100.0);
    }

    #[test]
    fn test_tech_list_completeness() {
        let list = get_tech_list();
        assert!(list.contains(&Tech::Masonry));
        assert!(list.contains(&Tech::MetalWorking));
        assert!(list.contains(&Tech::SocialStructures));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Create `src/ui/tech.rs`

```rust
use bevy_ecs::prelude::*;
use ratatui::prelude::*;
use crate::layer1::tech::Tech;

#[derive(Resource, Default, Debug)]
pub struct TechUiState {
    pub is_open: bool,
    pub selected_index: usize,
}

impl TechUiState {
    pub fn next(&mut self, count: usize) {
        if count == 0 { return; }
        self.selected_index = (self.selected_index + 1) % count;
    }

    pub fn prev(&mut self, count: usize) {
        if count == 0 { return; }
        self.selected_index = if self.selected_index == 0 {
            count - 1
        } else {
            self.selected_index - 1
        };
    }
}

pub fn get_tech_list() -> Vec<Tech> {
    vec![Tech::Masonry, Tech::MetalWorking, Tech::SocialStructures]
}

pub fn render_tech_tree(frame: &mut Frame, area: Rect, world: &World) {
    let ui_state = world.resource::<TechUiState>();
    if !ui_state.is_open {
        return;
    }

    // Render logic using ratatui::widgets::List or similar
    // Use Clear widget to overlay
}
```

### 2. Update `InputContext`

Add `TechTree` to `src/shared/input.rs`.

```rust
pub enum InputContext {
    // ...
    TechTree,
}
```

### 3. Update `InputRouter`

Handle toggling in `Normal` mode and navigation in `TechTree` mode.

```rust
// src/shared/input.rs

// In handle_normal_mode:
GameKeyCode::Char('t') => {
    world.resource_mut::<InputContextStack>().push(InputContext::TechTree);
    world.resource_mut::<crate::ui::tech::TechUiState>().is_open = true;
}

// New handler:
fn handle_tech_tree_mode(world: &mut World, key: GameKeyEvent) {
    let tech_count = crate::ui::tech::get_tech_list().len();

    match key.code {
        GameKeyCode::Esc | GameKeyCode::Char('t') => {
            world.resource_mut::<InputContextStack>().pop();
            world.resource_mut::<crate::ui::tech::TechUiState>().is_open = false;
        }
        GameKeyCode::Up | GameKeyCode::Char('w') => {
            world.resource_mut::<crate::ui::tech::TechUiState>().prev(tech_count);
        }
        GameKeyCode::Down | GameKeyCode::Char('s') => {
            world.resource_mut::<crate::ui::tech::TechUiState>().next(tech_count);
        }
        GameKeyCode::Enter | GameKeyCode::Char(' ') => {
            let ui_state = world.resource::<crate::ui::tech::TechUiState>();
            let idx = ui_state.selected_index;
            let techs = crate::ui::tech::get_tech_list();
            if idx < techs.len() {
                crate::layer1::tech::unlock_tech(world, techs[idx]);
            }
        }
        _ => {}
    }
}
```

### 4. Register Resource

Add `TechUiState` to `setup_world` in `src/setup.rs`.

### 5. Add Render Call

Update `src/ui/mod.rs` to call `render_tech_tree`.

## REFACTOR Phase: Quality & Design

- **Visuals**: Make the tech tree look nice. Use green for unlocked, red for unaffordable, white for available.
- **Scroll**: Use `ratatui::widgets::ListState` inside `TechUiState` if you want proper scrolling behavior (or just manual offset).
- **Architecture**: Keep UI logic in `ui/tech.rs`, separate from core logic.

## Acceptance Criteria

- [ ] `TechUiState` resource exists.
- [ ] Pressing `[T]` opens the Tech UI overlay.
- [ ] `[Esc]` closes it.
- [ ] Arrow keys navigate the list.
- [ ] `[Enter]` unlocks a selected tech if affordable.
- [ ] UI shows correct status (Unlocked/Cost).
- [ ] Tests pass.
