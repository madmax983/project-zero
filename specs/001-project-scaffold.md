# 001: Project Scaffold and Main Loop

## Overview

Set up the foundational application with bevy_ecs for simulation and ratatui for rendering. This is the skeleton everything else builds on.

## Dependencies

None — this is the first task.

## Requirements

### Must Have
- `Cargo.toml` with bevy_ecs, ratatui, crossterm dependencies
- Main loop: input → simulation tick → render
- Terminal setup with alternate screen, raw mode
- Clean shutdown on Ctrl+C or 'q'
- Basic frame with title "SCALE" and empty content area
- Module structure matching DESIGN.md architecture
- GameState resource: `Running`, `Paused`, `Quitting`

### Must NOT Have
- Any gameplay logic
- Any simulation systems
- Terrain or pops

## Technical Guidance

### Cargo.toml
```toml
[package]
name = "scale"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy_ecs = "0.15"
ratatui = "0.29"
crossterm = "0.28"

[profile.dev]
opt-level = 1
```

### Main Loop Structure

```rust
use bevy_ecs::prelude::*;
use ratatui::prelude::*;
use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    // Terminal setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
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
    
    Ok(())
}
```

### Resources

```rust
#[derive(Resource, Default, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Running,
    Paused,
    Quitting,
}
```

### Basic Render

```rust
fn render(world: &World, frame: &mut Frame) {
    let area = frame.area();
    
    let block = Block::default()
        .title(" SCALE ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    
    frame.render_widget(block, area);
}
```

### Input Handler

```rust
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
        _ => {}
    }
}
```

### Directory Structure
```
src/
  main.rs          # Main loop, terminal setup
  lib.rs           # Re-exports
  layer1/
    mod.rs         # pub mod declarations
  layer2/
    mod.rs
  layer3/
    mod.rs
  ui/
    mod.rs
  shared/
    mod.rs
```

## Acceptance Criteria

- [ ] `cargo build` succeeds
- [ ] `cargo run` shows terminal UI with "SCALE" title
- [ ] 'q' or Escape quits cleanly (terminal restored)
- [ ] Spacebar toggles pause (no visible effect yet, but state changes)
- [ ] Ctrl+C doesn't corrupt terminal
- [ ] Module structure exists
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
