# 012: Help Screen

## Overview

A reference screen that lists all available controls. This helps new players (and agents) understand how to interact with the game. It functions as a modal overlay, similar to the Chronicle.

## Dependencies

- `003` — UI layout (layout logic)

## Requirements

### Must Have
- Toggle with `?` key
- Modal overlay centered on screen
- Pauses the game while open (like Chronicle)
- Lists all current controls:
  - Movement: WASD / Arrows
  - Camera: WASD / Arrows (in normal mode)
  - Game Speed: 1, 2, 3, Space (Pause)
  - Build Mode: B (Toggle), Tab (Cycle), Enter (Place), Esc (Cancel)
  - Chronicle: L / H
  - Quit: Q
- Distinct title " Help / Controls "

### Must NOT Have
- Interactive elements (buttons)
- Pagination (fit on one screen)
- Configurable keybindings

## Technical Guidance

### Resources

```rust
#[derive(Resource, Default)]
pub struct HelpUiState {
    pub is_open: bool,
}
```

### Input System

```rust
fn toggle_help_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<HelpUiState>,
    mut sim_time: ResMut<SimulationTime>,
) {
    // Shift+? usually comes as KeyCode::Slash with modifiers, or just check '?' char if using char events.
    // Ideally use key code. KeyCode::Slash is usually '?'
    // But crossterm might report it differently.
    // For simplicity let's stick to KeyCode::Slash (which is usually ? on US keyboards) or check for '?' char in input handling if possible.
    // Given we use bevy input in systems, we might need KeyCode::Slash.

    if keys.just_pressed(KeyCode::Slash) {
        ui_state.is_open = !ui_state.is_open;

        if ui_state.is_open {
             sim_time.speed = SimSpeed::Paused;
        }
    }
}
```

### Rendering

Reuse modal logic from `010` (Chronicle) if possible, or duplicate for independence.

```rust
fn render_help(frame: &mut Frame, area: Rect, world: &World) {
    let ui_state = world.resource::<HelpUiState>();
    if !ui_state.is_open {
        return;
    }

    let block = Block::default()
        .title(" Help / Controls ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    // Helper to center rect (same as Chronicle)
    let area = centered_rect(50, 60, area);

    frame.render_widget(Clear, area);
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);

    let rows = vec![
        Row::new(vec!["General", ""]),
        Row::new(vec!["  Quit", "Q"]),
        Row::new(vec!["  Help", "?"]),
        Row::new(vec!["  Chronicle", "L or H"]),
        Row::new(vec!["", ""]),

        Row::new(vec!["Navigation", ""]),
        Row::new(vec!["  Pan View", "WASD / Arrows"]),
        Row::new(vec!["", ""]),

        Row::new(vec!["Time Control", ""]),
        Row::new(vec!["  Pause/Resume", "Space"]),
        Row::new(vec!["  Normal Speed", "1"]),
        Row::new(vec!["  Fast Speed", "2"]),
        Row::new(vec!["  Turbo Speed", "3"]),
        Row::new(vec!["", ""]),

        Row::new(vec!["Building", ""]),
        Row::new(vec!["  Toggle Mode", "B"]),
        Row::new(vec!["  Move Cursor", "WASD / Arrows"]),
        Row::new(vec!["  Cycle Type", "Tab"]),
        Row::new(vec!["  Place", "Enter"]),
        Row::new(vec!["  Cancel", "Esc"]),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)]
    );

    frame.render_widget(table, inner);
}
```

## Acceptance Criteria

- [ ] `HelpUiState` resource exists
- [ ] `?` (Slash) key toggles the Help window
- [ ] Window is centered and covers part of map
- [ ] Game auto-pauses when window opens
- [ ] All specified controls are listed
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
