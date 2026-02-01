# 003: UI Layout and Status Bar

## Overview

Establish the main UI layout with a map area, status bar, and info panel. This provides the visual framework for all future UI elements.

## Dependencies

- `001` — Project scaffold
- `002` — Terrain grid (need something to show in map area)

## Requirements

### Must Have
- Layout: map area (main), status bar (bottom), info panel (right)
- Status bar shows: game state (Running/Paused), tick count, speed indicator
- Info panel placeholder (empty for now, will show selection info)
- Map area fills remaining space
- Clean visual separation between areas

### Must NOT Have
- Interactive info panel content
- Multiple tabs or screens
- Minimap

## Technical Guidance

### Layout Structure

```
┌─────────────────────────────────────┬──────────────┐
│                                     │              │
│                                     │    Info      │
│              Map Area               │    Panel     │
│                                     │              │
│                                     │              │
├─────────────────────────────────────┴──────────────┤
│  ▶ Running  │  Tick: 142  │  Speed: 1x  │  ?:Help  │
└────────────────────────────────────────────────────┘
```

### Resources

```rust
#[derive(Resource)]
pub struct SimulationTime {
    pub tick: u64,
    pub speed: SimSpeed,
    pub accumulator: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SimSpeed {
    Paused,
    #[default]
    Normal,
    Fast,
    Faster,
}

impl SimSpeed {
    pub fn ticks_per_second(&self) -> f32 {
        match self {
            SimSpeed::Paused => 0.0,
            SimSpeed::Normal => 1.0,
            SimSpeed::Fast => 3.0,
            SimSpeed::Faster => 5.0,
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            SimSpeed::Paused => "⏸ Paused",
            SimSpeed::Normal => "▶ 1x",
            SimSpeed::Fast => "▶▶ 3x",
            SimSpeed::Faster => "▶▶▶ 5x",
        }
    }
}
```

### Layout Code

```rust
use ratatui::layout::{Constraint, Direction, Layout};

fn render(world: &World, frame: &mut Frame) {
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
    
    // Render map
    render_map(frame, map_area, world);
    
    // Render info panel
    render_info_panel(frame, info_area, world);
    
    // Render status bar
    render_status_bar(frame, status_area, world);
}

fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    // Render terrain inside
    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();
    render_terrain(frame, inner, terrain, viewport);
}

fn render_info_panel(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Info ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    // Placeholder text
    let text = Paragraph::new("Select something\nto see info here");
    frame.render_widget(text, inner);
}

fn render_status_bar(frame: &mut Frame, area: Rect, world: &World) {
    let sim_time = world.resource::<SimulationTime>();
    
    let status = format!(
        " {} │ Tick: {} │ {} │ WASD:Move  Space:Pause  1-3:Speed  q:Quit ",
        if sim_time.speed == SimSpeed::Paused { "⏸" } else { "▶" },
        sim_time.tick,
        sim_time.speed.label(),
    );
    
    let bar = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}
```

### Input Additions

```rust
KeyCode::Char('1') => {
    world.resource_mut::<SimulationTime>().speed = SimSpeed::Normal;
}
KeyCode::Char('2') => {
    world.resource_mut::<SimulationTime>().speed = SimSpeed::Fast;
}
KeyCode::Char('3') => {
    world.resource_mut::<SimulationTime>().speed = SimSpeed::Faster;
}
```

### Tick System

Note: The actual tick advancement will be part of the main loop timing, but the resource must exist:

```rust
// In main loop, after schedule.run():
if *world.resource::<GameState>() == GameState::Running {
    let speed = world.resource::<SimulationTime>().speed;
    if speed != SimSpeed::Paused {
        // Actual tick logic will be added by later specs
        world.resource_mut::<SimulationTime>().tick += 1;
    }
}
```

## Acceptance Criteria

- [ ] Three distinct UI areas visible: map, info panel, status bar
- [ ] Status bar shows current tick number
- [ ] Status bar shows speed indicator
- [ ] Speed changes with 1/2/3 keys
- [ ] Status bar updates when speed changes
- [ ] Paused state shows ⏸ icon
- [ ] `SimulationTime` resource exists and tracks ticks
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
