# 001: Project Scaffold and Main Loop

## Overview

Set up the foundational application with bevy_ecs for simulation and ratatui for rendering. This establishes the main loop structure (input → simulation → render), terminal handling, and module architecture that all future specs build upon.

## Dependencies

None — this is the first task.

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/main.rs - Test module at the end of file

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_gamestate_default() {
        let state = GameState::default();
        assert_eq!(state, GameState::Running);
    }

    #[test]
    fn test_gamestate_transitions() {
        // Test all valid state transitions
        let state = GameState::Running;
        assert_eq!(state, GameState::Running);

        let state = GameState::Paused;
        assert_eq!(state, GameState::Paused);

        let state = GameState::Quitting;
        assert_eq!(state, GameState::Quitting);
    }

    #[test]
    fn test_quit_input_q() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);

        handle_input(&mut world, key_event(KeyCode::Char('q')));

        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }

    #[test]
    fn test_quit_input_escape() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);

        handle_input(&mut world, key_event(KeyCode::Esc));

        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }

    #[test]
    fn test_pause_toggle_from_running() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);

        handle_input(&mut world, key_event(KeyCode::Char(' ')));

        assert_eq!(*world.resource::<GameState>(), GameState::Paused);
    }

    #[test]
    fn test_pause_toggle_from_paused() {
        let mut world = World::new();
        world.insert_resource(GameState::Paused);

        handle_input(&mut world, key_event(KeyCode::Char(' ')));

        assert_eq!(*world.resource::<GameState>(), GameState::Running);
    }

    #[test]
    fn test_pause_while_quitting_stays_quitting() {
        let mut world = World::new();
        world.insert_resource(GameState::Quitting);

        handle_input(&mut world, key_event(KeyCode::Char(' ')));

        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }

    #[test]
    fn test_unknown_input_ignored() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);

        handle_input(&mut world, key_event(KeyCode::Char('x')));

        // State should not change
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
    }

    #[test]
    fn test_gamestate_is_copy() {
        let state1 = GameState::Running;
        let state2 = state1; // Should copy, not move
        assert_eq!(state1, state2);
    }
}
```

**Test Coverage Requirements:**
- GameState resource: default value, all variants, transitions
- Input handling: quit keys (q, Esc), pause toggle (Space), unknown keys ignored
- State machine: Quitting state is terminal (doesn't transition)
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Cargo.toml

```toml
[package]
name = "scale"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy_ecs = "0.15"
ratatui = "0.29"
crossterm = "0.28"
anyhow = "1.0"

[profile.dev]
opt-level = 1  # Faster iteration while debugging

[profile.release]
lto = true
codegen-units = 1
```

### GameState Resource

```rust
// src/main.rs

use bevy_ecs::prelude::*;

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum GameState {
    #[default]
    Running,
    Paused,
    Quitting,
}
```

### Input Handler

```rust
// src/main.rs

use crossterm::event::{self, Event, KeyCode};

fn handle_input(world: &mut World, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            *world.resource_mut::<GameState>() = GameState::Quitting;
        }
        KeyCode::Char(' ') => {
            let mut state = world.resource_mut::<GameState>();
            *state = match *state {
                GameState::Running => GameState::Paused,
                GameState::Paused => GameState::Running,
                GameState::Quitting => GameState::Quitting,
            };
        }
        _ => {} // Ignore unknown keys
    }
}
```

### Main Loop

```rust
// src/main.rs

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders},
};
use std::io;
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    // Terminal setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ECS setup
    let mut world = World::new();
    world.insert_resource(GameState::Running);

    let mut schedule = Schedule::default();
    // Systems will be added here by other specs

    // Main loop
    let tick_rate = Duration::from_millis(100); // 10 FPS base
    let mut last_tick = Instant::now();

    loop {
        // Input
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                handle_input(&mut world, key);
            }
        }

        // Check quit
        if *world.resource::<GameState>() == GameState::Quitting {
            break;
        }

        // Simulation tick
        if last_tick.elapsed() >= tick_rate {
            schedule.run(&mut world);
            last_tick = Instant::now();
        }

        // Render
        terminal.draw(|frame| render(&world, frame))?;
    }

    // Cleanup
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn render(_world: &World, frame: &mut Frame) {
    let area = frame.area();

    let block = Block::default()
        .title(" SCALE ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    frame.render_widget(block, area);
}
```

### Module Structure

```rust
// src/lib.rs

pub mod layer1;
pub mod layer2;
pub mod layer3;
pub mod shared;
```

```rust
// src/layer1/mod.rs
// Empty for now - future specs will add modules

// src/layer2/mod.rs
// Empty for now

// src/layer3/mod.rs
// Empty for now

// src/shared/mod.rs
//! Shared resources and utilities.
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Magic numbers**: `Duration::from_millis(100)` and `(10)` are hardcoded
   - Future: Extract to constants or config file
   - Not critical for MVP

2. **Error handling**: `anyhow::Result` is catch-all
   - Future: Define specific error types for terminal setup failures
   - Current approach acceptable for scaffold

3. **Render function unused parameter**: `_world` not used yet
   - Will be used by future specs (terrain, pops, UI)
   - Suppress warning with underscore prefix

4. **Terminal setup duplication**: Setup and teardown are procedural
   - Future: Wrap in RAII guard struct
   - Current approach clear and explicit for MVP

### Performance Considerations

- **Input polling**: 10ms poll is responsive enough, doesn't peg CPU
- **Render every frame**: Acceptable for terminal UI (not GPU-bound)
- **Fixed timestep**: 100ms tick rate decoupled from frame rate (good)

### API Design Notes

- `GameState` is `Copy` - cheap to pass around, prevents borrow issues
- `handle_input` takes `&mut World` - allows future expansion for input context
- Module structure follows DESIGN.md three-layer architecture

### Future Extensibility

When adding new input handlers:
- Add new match arms to `handle_input`
- Future spec should define input routing/priority system
- Current flat match is fine for <10 keybinds

When adding systems:
- Use `schedule.add_systems()` in main before loop
- Future spec should define system ordering guarantees
- Current empty schedule is intentional

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for main.rs (excluding main loop/rendering)
- [x] `cargo run` shows terminal UI with "SCALE" title
- [x] 'q' or Escape quits cleanly (terminal restored)
- [x] Spacebar toggles pause (state changes verified via tests)
- [x] Ctrl+C doesn't corrupt terminal (crossterm handles SIGINT)
- [x] Module structure exists (layer1, layer2, layer3, shared)
- [x] GameState has Debug derive for test assertions

## Technical Guidance

### Directory Structure

```
scale/
├── Cargo.toml
├── src/
│   ├── main.rs          # Main loop, terminal setup, input, render
│   ├── lib.rs           # Module re-exports
│   ├── layer1/
│   │   └── mod.rs       # Colony simulation (empty for now)
│   ├── layer2/
│   │   └── mod.rs       # System simulation (empty for now)
│   ├── layer3/
│   │   └── mod.rs       # Galaxy simulation (empty for now)
│   └── shared/
│       └── mod.rs       # Shared resources/utilities
```

### Integration with Future Specs

**Spec 002 (Terrain)** will:
- Add `use scale::layer1::*` imports
- Insert TerrainGrid resource in main
- Call terrain rendering in `render()`

**Spec 003 (UI Layout)** will:
- Replace simple `render()` with multi-panel layout
- Add more key bindings to `handle_input()`

**System-adding specs** will:
- Import system functions
- Call `schedule.add_systems()` before main loop

### Common Pitfalls

1. **Forgetting terminal cleanup**: Always use `?` in main, cleanup in Drop would be better
2. **Raw mode persists after panic**: Consider `panic::catch_unwind` or terminal guard
3. **Event polling blocking**: Never call `event::read()` without `event::poll()` first
4. **Schedule runs every tick**: Empty schedule is fine, has negligible cost

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
