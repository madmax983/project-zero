# 026: Main Menu

## Overview

Implement a Main Menu state that acts as the entry point for the game. Instead of launching directly into the simulation, the game should start in a menu with options to "Start Game" or "Quit".

This adds:
1.  **GameState::MainMenu**: A new state where simulation is paused/not started.
2.  **InputContext::MainMenu**: Input handling for menu navigation.
3.  **UI**: A visual menu rendering overlay.

## Dependencies

- `001` — Project scaffold (for `GameState`)
- `012` — Input Architecture (for `InputContext`, `InputRouter`)
- `003` — UI Layout (for `ratatui` integration)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/shared/state_tests.rs or similar

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::shared::state::GameState;
    use crate::shared::input::{InputContext, InputContextStack, InputRouter};
    use crate::ui::menu::MenuState; // New component/resource
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_gamestate_main_menu_variant() {
        let state = GameState::MainMenu;
        assert_ne!(state, GameState::Running);
        assert_ne!(state, GameState::Paused);
    }

    #[test]
    fn test_input_context_main_menu_variant() {
        let ctx = InputContext::MainMenu;
        assert_ne!(ctx, InputContext::Normal);
    }

    #[test]
    fn test_menu_state_resource() {
        let menu = MenuState::default();
        // Should default to first option (Start Game)
        assert_eq!(menu.selected_index, 0);
        // Should have options
        assert!(menu.options.len() >= 2);
        assert_eq!(menu.options[0], "Start Game");
        assert_eq!(menu.options[1], "Quit");
    }

    #[test]
    fn test_menu_navigation_down() {
        let mut menu = MenuState::default();
        menu.selected_index = 0;

        menu.next();
        assert_eq!(menu.selected_index, 1);

        // Should wrap or clamp (design decision: clamp)
        menu.next();
        assert_eq!(menu.selected_index, 1); // Assuming 2 options
    }

    #[test]
    fn test_menu_navigation_up() {
        let mut menu = MenuState::default();
        menu.selected_index = 1;

        menu.prev();
        assert_eq!(menu.selected_index, 0);

        menu.prev();
        assert_eq!(menu.selected_index, 0); // Clamp
    }

    #[test]
    fn test_menu_input_start_game() {
        let mut world = World::new();
        world.insert_resource(GameState::MainMenu);

        let mut stack = InputContextStack::default();
        // Force stack to MainMenu (in Green phase, this might be default)
        // For test, we can just push it or verify default
        // stack.push(InputContext::MainMenu);
        world.insert_resource(stack);
        // We need to ensure InputContextStack::default() or setup puts us in MainMenu if we change default
        // For now, assume we manually set it for the test
        world.resource_mut::<InputContextStack>().push(InputContext::MainMenu);

        world.insert_resource(MenuState { selected_index: 0, ..Default::default() }); // "Start Game" selected

        let mut router = InputRouter::new();
        router.route(&mut world, key_event(KeyCode::Enter));

        // Should transition to Running
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
        // Should switch input context to Normal
        assert_eq!(world.resource::<InputContextStack>().current(), InputContext::Normal);
    }

    #[test]
    fn test_menu_input_quit() {
        let mut world = World::new();
        world.insert_resource(GameState::MainMenu);
        world.insert_resource(InputContextStack::default());
        world.resource_mut::<InputContextStack>().push(InputContext::MainMenu);

        world.insert_resource(MenuState { selected_index: 1, ..Default::default() }); // "Quit" selected

        let mut router = InputRouter::new();
        router.route(&mut world, key_event(KeyCode::Enter));

        // Should transition to Quitting
        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update GameState

```rust
// src/shared/state.rs
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum GameState {
    #[default]
    MainMenu, // Change default from Running
    Running,
    Paused,
    Quitting,
}
```

### 2. Update InputContext

```rust
// src/shared/input.rs
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum InputContext {
    #[default]
    MainMenu, // Change default from Normal
    Normal,
    // ...
}
```

### 3. Implement MenuState Resource

```rust
// src/ui/menu.rs (New File)
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct MenuState {
    pub selected_index: usize,
    pub options: Vec<String>,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            options: vec!["Start Game".to_string(), "Quit".to_string()],
        }
    }
}

impl MenuState {
    pub fn next(&mut self) {
        if self.selected_index < self.options.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
}
```

### 4. Update InputRouter

```rust
// src/shared/input.rs

// In route():
match context {
    InputContext::MainMenu => handle_main_menu_mode(world, key),
    // ...
}

fn handle_main_menu_mode(world: &mut World, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('w') => {
            world.resource_mut::<MenuState>().prev();
        }
        KeyCode::Down | KeyCode::Char('s') => {
            world.resource_mut::<MenuState>().next();
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            let selected = world.resource::<MenuState>().selected_index;
            match selected {
                0 => { // Start Game
                    *world.resource_mut::<GameState>() = GameState::Running;
                    // Reset stack to Normal
                    let mut stack = world.resource_mut::<InputContextStack>();
                    // Assuming stack is [MainMenu], we want [Normal]
                    // Or if MainMenu is bottom, replace it.
                    // For MVP: stack.stack = vec![InputContext::Normal];
                    // Or push Normal on top? If we push Normal, ESC popping might return to Menu.
                    // Ideally: Main Menu is root. Start Game pushes Normal?
                    // Let's say MainMenu is root. "Start" replaces it or pushes Normal.
                    // Pushing Normal allows "Quit to Menu" later.
                    stack.push(InputContext::Normal);
                }
                1 => { // Quit
                    *world.resource_mut::<GameState>() = GameState::Quitting;
                }
                _ => {}
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
             *world.resource_mut::<GameState>() = GameState::Quitting;
        }
        _ => {}
    }
}
```

### 5. Render Main Menu

```rust
// src/ui/menu.rs

use ratatui::prelude::*;
use crate::shared::state::GameState;

pub fn render_main_menu(frame: &mut Frame, area: Rect, state: &MenuState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(10), // Menu items
            Constraint::Min(10),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("SCALE")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, layout[0]);

    // Options
    let constraints: Vec<Constraint> = state.options.iter().map(|_| Constraint::Length(1)).collect();
    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(layout[1]);

    for (i, option) in state.options.iter().enumerate() {
        let style = if i == state.selected_index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let text = if i == state.selected_index {
            format!("> {}", option)
        } else {
            format!("  {}", option)
        };

        let p = Paragraph::new(text).style(style).alignment(Alignment::Center);
        frame.render_widget(p, menu_layout[i]);
    }
}
```

### 6. Update Main Loop

```rust
// src/main.rs

fn run_app(...) {
    // ...
    world.insert_resource(MenuState::default()); // Add resource
    // GameState default is now MainMenu
    // InputContextStack default is now MainMenu
    // ...

    loop {
        // ...

        // Render
        terminal.draw(|frame| render(&world, frame))?;
    }
}

fn render(world: &World, frame: &mut Frame) {
    let state = world.resource::<GameState>();
    if *state == GameState::MainMenu {
        let menu_state = world.resource::<MenuState>();
        use scale::ui::menu::render_main_menu;
        render_main_menu(frame, frame.area(), menu_state);
    } else {
        // Existing render logic
        // ...
    }
}
```

## REFACTOR Phase: Quality & Design

- **InputContext Management**: Switching from `MainMenu` to `Normal` needs to be robust. Using `stack.push(Normal)` works if we want to support "Back to Menu".
- **Module Structure**: Ensure `menu.rs` is mod-ed in `src/ui/mod.rs`.
- **Styling**: Improve title art (maybe ASCII art?).

## Acceptance Criteria

- [ ] `GameState` defaults to `MainMenu`.
- [ ] Game starts with Main Menu visible.
- [ ] Arrow keys / W/S navigate menu options.
- [ ] Enter selects option.
- [ ] "Start Game" enters `Running` state and `Normal` input context.
- [ ] "Quit" exits the application.
- [ ] Tests pass.

## Technical Guidance

- In `src/main.rs`, ensuring `InputContextStack` initializes correctly is key. If `InputContext::default()` is changed to `MainMenu`, `InputContextStack::default()` will naturally start with it.
- Be careful with `InputContext` import cycles if moving `MenuState` around. `MenuState` belongs in `ui` or `layer1`, but it drives logic. Putting it in `ui/menu.rs` is fine if `input.rs` can see it. `input.rs` is in `shared`, so it cannot depend on `ui`.
- **Architectural Adjustment**: `MenuState` should probably be in `shared/state.rs` or a new `shared/menu.rs` so `InputRouter` (in `shared`) can access it without circular dependency on `ui`.
- **Decision**: Put `MenuState` in `src/shared/state.rs` (or `src/shared/menu.rs`) to avoid `shared -> ui` dependency. Rendering remains in `ui`.

## Questions

*Builder: Add questions here.*
*Architect: This spec is currently self-contained. Questions from future builders will be answered here.*
*Architect: No outstanding questions.*
