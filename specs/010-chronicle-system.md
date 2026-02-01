# 010: Chronicle System

## Overview

The Chronicle is the historical record of the colony. It records major events like foundation, milestones, and disasters. This system establishes the `Chronicle` resource, the UI to view it, and the first few automatic events.

## Dependencies

- `003` — UI layout (need to render overlay)
- `006` — Building placement (to trigger milestones)

## Requirements

### Must Have
- `Chronicle` resource storing a list of events
- `ChronicleEvent` struct with year/tick, text, and importance
- "Colony Founded" event generated at game start
- "Building Milestone" event generated when the *first* building of a type is completed
- UI window:
    - Toggled by `L` key (Log) or `H` key (History)
    - Renders as a centered modal overlay
    - Lists events in reverse chronological order (newest top)
    - Shows generic "Chronicle" title
- Pause game when Chronicle is open

### Must NOT Have
- Complex filtering or searching
- Persistent storage (save/load) — MVP is session-only
- Procedural text generation (use static strings for now, or simple formatting)

## Technical Guidance

### Data Structures

```rust
#[derive(Resource, Default)]
pub struct Chronicle {
    pub events: Vec<ChronicleEvent>,
}

#[derive(Clone, Debug)]
pub struct ChronicleEvent {
    pub tick: u64,
    pub year: u32,  // Derived from tick or 1 for MVP
    pub text: String,
    pub importance: EventImportance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventImportance {
    Minor,
    Standard,
    Major,
    Legendary,
}

impl Chronicle {
    pub fn add_event(&mut self, tick: u64, text: String, importance: EventImportance) {
        self.events.push(ChronicleEvent {
            tick,
            year: 1 + (tick / 1000) as u32, // Rough "year" approximation
            text,
            importance,
        });
    }
}
```

### UI Resources

```rust
#[derive(Resource, Default)]
pub struct ChronicleUiState {
    pub is_open: bool,
}
```

### Input System

```rust
fn toggle_chronicle_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>, // Note: ButtonInput in Bevy 0.13+
    mut ui_state: ResMut<ChronicleUiState>,
    mut sim_time: ResMut<SimulationTime>,
) {
    if keys.just_pressed(KeyCode::KeyL) || keys.just_pressed(KeyCode::KeyH) {
        ui_state.is_open = !ui_state.is_open;

        // Auto-pause when opening, maybe unpause when closing (or restore prev state)
        // For MVP, just toggle Pause if opening
        if ui_state.is_open {
             sim_time.speed = SimSpeed::Paused;
        }
    }
}
```

### Rendering

Use a centered Rect for the modal.

```rust
fn render_chronicle(frame: &mut Frame, area: Rect, world: &World) {
    let ui_state = world.resource::<ChronicleUiState>();
    if !ui_state.is_open {
        return;
    }

    let chronicle = world.resource::<Chronicle>();

    // Clear background of modal
    let block = Block::default()
        .title(" Chronicle ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    // Helper to center rect
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    let area = centered_rect(60, 60, area);
    frame.render_widget(Clear, area); // Clear underneath
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);

    let items: Vec<ListItem> = chronicle.events.iter().rev().map(|evt| {
        let prefix = match evt.importance {
            EventImportance::Legendary => "!!!",
            EventImportance::Major => "!",
            _ => " ",
        };
        // Simple formatting: [123] Y1: Message
        ListItem::new(format!(
            "[{}] Y{}: {} {}",
            evt.tick, evt.year, prefix, evt.text
        ))
    }).collect();

    let list = List::new(items)
        .block(Block::default());

    frame.render_widget(list, inner);
}
```

### Event Triggers

**Startup:**
```rust
fn initial_chronicle_event(mut chronicle: ResMut<Chronicle>) {
    chronicle.add_event(0, "Colony founded. The journey begins.".to_string(), EventImportance::Legendary);
}
```

**Milestones:**
Need a resource to track what has been built.

```rust
#[derive(Resource, Default)]
pub struct BuildingTracker {
    pub has_built_housing: bool,
    pub has_built_farm: bool,
}

fn check_milestones_system(
    mut chronicle: ResMut<Chronicle>,
    mut tracker: ResMut<BuildingTracker>,
    query: Query<&Building>,
    time: Res<SimulationTime>,
) {
    for building in query.iter() {
        match building.building_type {
            BuildingType::Housing => {
                if !tracker.has_built_housing {
                    tracker.has_built_housing = true;
                    chronicle.add_event(
                        time.tick,
                        "First Housing constructed. A shelter from the void.".to_string(),
                        EventImportance::Major
                    );
                }
            }
            BuildingType::Farm => {
                if !tracker.has_built_farm {
                    tracker.has_built_farm = true;
                    chronicle.add_event(
                        time.tick,
                        "First Farm operational. We shall not starve.".to_string(),
                        EventImportance::Major
                    );
                }
            }
        }
    }
}
```

## Acceptance Criteria

- [ ] `Chronicle` resource exists
- [ ] `L` or `H` key toggles the Chronicle window
- [ ] Window is centered and covers part of map
- [ ] Game auto-pauses when window opens
- [ ] "Colony founded" event appears at start
- [ ] Building a Farm triggers a milestone event
- [ ] Building Housing triggers a milestone event
- [ ] Events show tick/year and text
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
