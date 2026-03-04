# 003: UI Layout and Status Bar

## Overview

Establish the main UI layout with three panels (map, info, status bar) and add simulation time tracking. This creates the visual framework for all future UI elements and introduces the tick counter that will drive needs decay, job progress, and other time-based systems.

## Dependencies

- `001` — Project scaffold
- `002` — Terrain grid (need something to show in map area)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/shared/time.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_speed_default() {
        let speed = SimSpeed::default();
        assert_eq!(speed, SimSpeed::Normal);
    }

    #[test]
    fn test_sim_speed_values() {
        // Use epsilon comparison for floats
        assert!((SimSpeed::Paused.ticks_per_second() - 0.0).abs() < f32::EPSILON);
        assert!((SimSpeed::Normal.ticks_per_second() - 1.0).abs() < f32::EPSILON);
        assert!((SimSpeed::Fast.ticks_per_second() - 3.0).abs() < f32::EPSILON);
        assert!((SimSpeed::Faster.ticks_per_second() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sim_speed_labels() {
        assert!(SimSpeed::Paused.label().contains("Paused"));
        assert!(SimSpeed::Normal.label().contains("1x"));
        assert!(SimSpeed::Fast.label().contains("3x"));
        assert!(SimSpeed::Faster.label().contains("5x"));
    }

    #[test]
    fn test_simulation_time_default() {
        let sim_time = SimulationTime::default();
        assert_eq!(sim_time.tick, 0);
        assert_eq!(sim_time.speed, SimSpeed::Normal);
        assert!((sim_time.accumulator - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_simulation_time_tick_increment() {
        let mut sim_time = SimulationTime::default();
        assert_eq!(sim_time.tick, 0);

        sim_time.tick += 1;
        assert_eq!(sim_time.tick, 1);

        sim_time.tick += 10;
        assert_eq!(sim_time.tick, 11);
    }

    #[test]
    fn test_sim_speed_transitions() {
        let mut speed = SimSpeed::Normal;
        assert_eq!(speed, SimSpeed::Normal);

        speed = SimSpeed::Fast;
        assert_eq!(speed, SimSpeed::Fast);

        speed = SimSpeed::Paused;
        assert_eq!(speed, SimSpeed::Paused);

        speed = SimSpeed::Faster;
        assert_eq!(speed, SimSpeed::Faster);
    }

    #[test]
    fn test_speed_is_copy() {
        let speed1 = SimSpeed::Fast;
        let speed2 = speed1; // Should copy, not move
        assert_eq!(speed1, speed2);
    }

    #[test]
    fn test_speed_persists_across_tick() {
        let mut sim_time = SimulationTime::default();
        sim_time.speed = SimSpeed::Fast;

        sim_time.tick += 1;

        // Speed should not change when tick increments
        assert_eq!(sim_time.speed, SimSpeed::Fast);
    }

    #[test]
    fn test_accumulator_field_exists() {
        // Accumulator exists but is unused in MVP
        // Reserved for future speed multiplier implementation
        let mut sim_time = SimulationTime::default();
        assert!((sim_time.accumulator - 0.0).abs() < f32::EPSILON);

        sim_time.accumulator = 0.5;
        assert!((sim_time.accumulator - 0.5).abs() < f32::EPSILON);
    }
}
```

```rust
// src/main.rs - Add to existing test module

#[test]
fn test_speed_key_1() {
    let mut world = create_test_world();
    world.insert_resource(SimulationTime::default());
    world.resource_mut::<SimulationTime>().speed = SimSpeed::Fast;

    handle_input(&mut world, key_event(KeyCode::Char('1')));

    assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Normal);
}

#[test]
fn test_speed_key_2() {
    let mut world = create_test_world();
    world.insert_resource(SimulationTime::default());

    handle_input(&mut world, key_event(KeyCode::Char('2')));

    assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Fast);
}

#[test]
fn test_speed_key_3() {
    let mut world = create_test_world();
    world.insert_resource(SimulationTime::default());

    handle_input(&mut world, key_event(KeyCode::Char('3')));

    assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Faster);
}

#[test]
fn test_tick_increments_when_running() {
    let mut world = create_test_world();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(GameState::Running);

    // Simulate tick logic from main loop
    if *world.resource::<GameState>() == GameState::Running {
        let speed = world.resource::<SimulationTime>().speed;
        if speed != SimSpeed::Paused {
            world.resource_mut::<SimulationTime>().tick += 1;
        }
    }

    assert_eq!(world.resource::<SimulationTime>().tick, 1);
}

#[test]
fn test_tick_does_not_increment_when_paused_gamestate() {
    let mut world = create_test_world();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(GameState::Paused);

    // Simulate tick logic from main loop
    if *world.resource::<GameState>() == GameState::Running {
        world.resource_mut::<SimulationTime>().tick += 1;
    }

    assert_eq!(world.resource::<SimulationTime>().tick, 0);
}
```

**Test Coverage Requirements:**
- SimSpeed: all variants, labels, ticks_per_second values
- SimulationTime: default state, tick increment, speed changes
- Input handling: speed keys (1, 2, 3)
- Tick logic: increments when running, paused blocks ticks
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### SimulationTime Resource

```rust
// src/shared/time.rs

use bevy_ecs::prelude::*;

/// Tracks the global simulation time and speed.
#[derive(Resource, Default)]
pub struct SimulationTime {
    /// The current simulation tick (update count).
    pub tick: u64,

    /// The current simulation speed setting.
    pub speed: SimSpeed,

    /// Accumulator for partial ticks when running at non-integer speeds.
    ///
    /// **NOTE (MVP):** Currently unused. Reserved for future implementation where speed
    /// multipliers will be applied via fractional tick accumulation. For now, all
    /// speeds increment by 1 per frame. Future spec will implement actual speed multiplier.
    pub accumulator: f32,
}

/// Defines the speed at which the simulation runs.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SimSpeed {
    /// The simulation is paused (via speed control, not GameState).
    /// **NOTE:** Currently no key binding sets this. Use spacebar for GameState::Paused.
    Paused,

    /// The simulation runs at 1x speed.
    #[default]
    Normal,

    /// The simulation runs at 3x speed (future implementation).
    Fast,

    /// The simulation runs at 5x speed (future implementation).
    Faster,
}

impl SimSpeed {
    /// Returns the number of ticks per second for this speed.
    ///
    /// **NOTE (MVP):** Currently unused in the main game loop. The tick increment logic
    /// always adds 1 per tick regardless of speed setting. This method is reserved for
    /// future implementation where the speed multiplier will be applied via the
    /// `accumulator` field. Future spec will implement actual tick rate differences.
    #[must_use]
    pub const fn ticks_per_second(&self) -> f32 {
        match self {
            Self::Paused => 0.0,
            Self::Normal => 1.0,
            Self::Fast => 3.0,
            Self::Faster => 5.0,
        }
    }

    /// Returns a user-friendly label for the UI.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Paused => "⏸ Paused",
            Self::Normal => "▶ 1x",
            Self::Fast => "▶▶ 3x",
            Self::Faster => "▶▶▶ 5x",
        }
    }
}
```

### Module Integration

```rust
// src/shared/mod.rs
//! Shared resources and utilities.

/// Time-tracking resources.
pub mod time;
```

```rust
// src/lib.rs - Add to existing exports
pub mod shared;
```

### Main.rs Changes

```rust
// src/main.rs - Add imports

use scale::shared::time::{SimSpeed, SimulationTime};

// Modify render() function - split into three panels:

fn render(world: &World, frame: &mut Frame) {
    use ratatui::layout::{Constraint, Direction, Layout};

    // Main vertical split: content + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),     // Content area
            Constraint::Length(1),   // Status bar
        ])
        .split(frame.area());

    // Horizontal split: map + info panel
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),      // Map area
            Constraint::Length(20),   // Info panel
        ])
        .split(main_chunks[0]);

    let map_area = content_chunks[0];
    let info_area = content_chunks[1];
    let status_area = main_chunks[1];

    // Render three panels
    render_map(frame, map_area, world);
    render_info_panel(frame, info_area, world);
    render_status_bar(frame, status_area, world);
}

fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render terrain inside
    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();
    render_terrain(frame, inner, terrain, viewport);
}

fn render_info_panel(frame: &mut Frame, area: Rect, _world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Info ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Placeholder text
    let text = Paragraph::new("Select something\nto see info here");
    frame.render_widget(text, inner);
}

fn render_status_bar(frame: &mut Frame, area: Rect, world: &World) {
    let sim_time = world.resource::<SimulationTime>();
    let game_state = world.resource::<GameState>();

    // NOTE: Dual pause state check. GameState::Paused is controlled by spacebar,
    // SimSpeed::Paused exists but is currently not used (no key binds to it).
    // This allows for future distinction between "paused but still simulating at 0x"
    // vs "completely frozen". Current behavior: only GameState::Paused matters.
    let paused = *game_state == GameState::Paused || sim_time.speed == SimSpeed::Paused;

    let status = format!(
        " {} │ Tick: {} │ {} │ WASD:Move  Space:Pause  1-3:Speed  q:Quit ",
        if paused { "⏸" } else { "▶" },
        sim_time.tick,
        sim_time.speed.label(),
    );

    let bar = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}

// In main() after world.insert_resource(Viewport::default()):
world.insert_resource(SimulationTime::default());

// In handle_input(), add speed control:
KeyCode::Char('1') => {
    world.resource_mut::<SimulationTime>().speed = SimSpeed::Normal;
}
KeyCode::Char('2') => {
    world.resource_mut::<SimulationTime>().speed = SimSpeed::Fast;
}
KeyCode::Char('3') => {
    world.resource_mut::<SimulationTime>().speed = SimSpeed::Faster;
}

// In main loop, after schedule.run():
if last_tick.elapsed() >= tick_rate {
    schedule.run(&mut world);

    // Update tick count (MVP: always increments by 1, speed visual only)
    if *world.resource::<GameState>() == GameState::Running {
        let speed = world.resource::<SimulationTime>().speed;
        if speed != SimSpeed::Paused {
            world.resource_mut::<SimulationTime>().tick += 1;
        }
    }

    last_tick = Instant::now();
}
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Speed multiplier not implemented**: UI shows "1x/3x/5x" but all tick at same rate
   - Future spec needed: "Speed Multiplier System"
   - Would use `accumulator` and `ticks_per_second()` to apply actual speed differences
   - Current cosmetic-only implementation acceptable for MVP

2. **Dual pause state confusion**: Both `GameState::Paused` and `SimSpeed::Paused` exist
   - GameState::Paused = spacebar (working)
   - SimSpeed::Paused = no key binding (unused)
   - Future: Either remove SimSpeed::Paused or bind it to a key
   - Current: Document the distinction for future expansion

3. **Layout magic numbers**: Panel widths (20 for info) hardcoded
   - Future: Make configurable or responsive
   - Current values work well for most terminals

4. **Status bar format string**: Complex with many symbols
   - Future: Extract to builder pattern or template
   - Current readable and explicit

5. **Info panel always empty**: Placeholder text only
   - Future specs (selection system) will populate
   - Current placeholder communicates intent

### Performance Considerations

- **Three-panel layout**: Negligible overhead, ratatui handles efficiently
- **Status bar updates every frame**: Cheap string formatting, acceptable
- **Dual pause check**: Two comparisons per frame, trivial cost

### API Design Notes

- `SimulationTime` has public fields - acceptable for resources
- `SimSpeed` is Copy - cheap to pass, no borrow issues
- Tick counter is u64 - won't overflow for billions of years of gameplay
- Layout split values chosen for readability, not performance

### Future Extensibility

When implementing speed multiplier (future spec):
```rust
// Use accumulator for fractional ticks
sim_time.accumulator += delta_time * speed.ticks_per_second();
while sim_time.accumulator >= 1.0 {
    sim_time.tick += 1;
    sim_time.accumulator -= 1.0;
    // Run simulation systems
}
```

When adding selection system (future spec):
- Populate info panel with selected entity details
- Show pop stats, building info, tile terrain type
- Add cursor/selection state to Viewport or new resource

When adding more speed options:
- Add SimSpeed::SuperFast (10x), UltraFast (100x)
- Consider adding pause icon to status bar independently

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for shared/time.rs
- [x] Three distinct UI areas visible: map, info panel, status bar
- [x] Status bar shows current tick number (starts at 0)
- [x] Status bar shows speed indicator (default "▶ 1x")
- [x] Speed changes with 1/2/3 keys (verified via tests)
- [x] Status bar updates when speed changes
- [x] Paused state shows ⏸ icon (GameState::Paused)
- [x] SimulationTime resource exists and tracks ticks
- [x] Speed is cosmetic only (all speeds tick at 1x for MVP)

## Technical Guidance

### Layout Constraints

```
┌─────────────────────────────────────┬──────────────┐
│                                     │              │
│                                     │    Info      │
│              Map Area               │    Panel     │  Constraint::Min(10)
│      (Constraint::Min(20))          │   (Len 20)   │
│                                     │              │
├─────────────────────────────────────┴──────────────┤
│  Status Bar (Constraint::Length(1))                 │
└────────────────────────────────────────────────────┘
```

- Map area: Flexible, takes remaining width/height
- Info panel: Fixed 20 columns wide
- Status bar: Fixed 1 row tall

### Tick Increment Logic

**Current (MVP):**
```rust
// All speeds increment by 1
if running && speed != Paused {
    tick += 1;
}
```

**Future (with speed multiplier spec):**
```rust
// Accumulate fractional ticks
accumulator += delta_time * speed.ticks_per_second();
while accumulator >= 1.0 {
    tick += 1;
    accumulator -= 1.0;
}
```

### Info Panel Future Population

When selection system is implemented, info panel will show:
- **No selection**: "Select something to see info here" (current)
- **Pop selected**: Name, needs bars, current job
- **Building selected**: Type, workers, production status
- **Tile selected**: Terrain type, resources, designations

### Common Pitfalls

1. **Assuming speed works**: UI shows 3x but ticks at 1x - this is intentional for MVP
2. **Confusing pause states**: Spacebar = GameState, not SimSpeed
3. **Forgetting inner_area**: Render terrain inside block.inner(), not full area
4. **Status bar too wide**: Long help text may wrap on narrow terminals

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

## Future Work

This spec intentionally leaves unimplemented:
- **Speed Multiplier System** - Apply actual tick rate differences using accumulator
- **Selection System** - Populate info panel with entity/tile details
- **SimSpeed::Paused key binding** - Currently unused, could bind to 'P' key
- **Responsive layout** - Adjust panel sizes based on terminal width
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
