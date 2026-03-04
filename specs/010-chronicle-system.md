# 010: Chronicle System

## Overview

The Chronicle is the historical record of the colony. It records major events like foundation, milestones, and disasters. This system establishes the `Chronicle` resource, the UI to view it, and the first few automatic events.

## Dependencies

- `003` — UI layout (need to render overlay)
- `006` — Building placement (to trigger milestones)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer3/chronicle.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_event_importance_variants() {
        // Just verify all variants exist
        let _ = EventImportance::Minor;
        let _ = EventImportance::Standard;
        let _ = EventImportance::Major;
        let _ = EventImportance::Legendary;
    }

    #[test]
    fn test_chronicle_event_creation() {
        let event = ChronicleEvent {
            tick: 100,
            year: 1,
            text: "Test event".to_string(),
            importance: EventImportance::Standard,
        };

        assert_eq!(event.tick, 100);
        assert_eq!(event.year, 1);
        assert_eq!(event.text, "Test event");
        assert_eq!(event.importance, EventImportance::Standard);
    }

    #[test]
    fn test_chronicle_default() {
        let chronicle = Chronicle::default();
        assert!(chronicle.events.is_empty());
    }

    #[test]
    fn test_chronicle_add_event() {
        let mut chronicle = Chronicle::default();

        chronicle.add_event(0, "First event".to_string(), EventImportance::Legendary);
        chronicle.add_event(100, "Second event".to_string(), EventImportance::Standard);

        assert_eq!(chronicle.events.len(), 2);
        assert_eq!(chronicle.events[0].text, "First event");
        assert_eq!(chronicle.events[1].text, "Second event");
    }

    #[test]
    fn test_chronicle_year_calculation() {
        let mut chronicle = Chronicle::default();

        chronicle.add_event(0, "Year 1".to_string(), EventImportance::Standard);
        chronicle.add_event(1000, "Year 2".to_string(), EventImportance::Standard);
        chronicle.add_event(5000, "Year 6".to_string(), EventImportance::Standard);

        assert_eq!(chronicle.events[0].year, 1);
        assert_eq!(chronicle.events[1].year, 2);
        assert_eq!(chronicle.events[2].year, 6);
    }

    #[test]
    fn test_chronicle_ui_state_default() {
        let ui_state = ChronicleUiState::default();
        assert!(!ui_state.is_open);
    }

    #[test]
    fn test_chronicle_ui_state_toggle() {
        let mut ui_state = ChronicleUiState::default();
        assert!(!ui_state.is_open);

        ui_state.is_open = true;
        assert!(ui_state.is_open);

        ui_state.is_open = !ui_state.is_open;
        assert!(!ui_state.is_open);
    }

    #[test]
    fn test_building_tracker_default() {
        let tracker = BuildingTracker::default();
        assert!(!tracker.has_built_housing);
        assert!(!tracker.has_built_farm);
    }

    #[test]
    fn test_check_milestones_system_housing() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());
        world.insert_resource(BuildingTracker::default());
        world.insert_resource(SimulationTime::default());

        // Place housing
        world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 5, y: 5 },
        ));

        check_milestones_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("Housing"));

        let tracker = world.resource::<BuildingTracker>();
        assert!(tracker.has_built_housing);
    }

    #[test]
    fn test_check_milestones_system_farm() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());
        world.insert_resource(BuildingTracker::default());
        world.insert_resource(SimulationTime::default());

        // Place farm
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
        ));

        check_milestones_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("Farm"));

        let tracker = world.resource::<BuildingTracker>();
        assert!(tracker.has_built_farm);
    }

    #[test]
    fn test_check_milestones_system_only_once() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());
        world.insert_resource(BuildingTracker::default());
        world.insert_resource(SimulationTime::default());

        // Place two farms
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
        ));
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 10, y: 10 },
        ));

        check_milestones_system(&mut world);
        check_milestones_system(&mut world); // Run twice

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1, "Should only record first farm");
    }

    #[test]
    fn test_initial_chronicle_event() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());

        initial_chronicle_event(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("founded"));
        assert_eq!(chronicle.events[0].importance, EventImportance::Legendary);
    }

    #[test]
    fn test_format_event_prefix() {
        assert_eq!(format_event_prefix(EventImportance::Legendary), "!!!");
        assert_eq!(format_event_prefix(EventImportance::Major), "!");
        assert_eq!(format_event_prefix(EventImportance::Standard), " ");
        assert_eq!(format_event_prefix(EventImportance::Minor), " ");
    }
}
```

**Test Coverage Requirements:**
- EventImportance: all variants
- ChronicleEvent: creation, fields
- Chronicle: default, add_event, year calculation
- ChronicleUiState: default, toggle
- BuildingTracker: default, milestone tracking
- check_milestones_system: triggers for housing/farm, only once per type
- initial_chronicle_event: creates foundation event
- format_event_prefix: importance indicators
- Coverage ≥85% for layer3/chronicle.rs
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Data Structures

```rust
// src/layer3/chronicle.rs

use bevy_ecs::prelude::*;

/// Importance level for chronicle events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventImportance {
    Minor,
    Standard,
    Major,
    Legendary,
}

/// A single chronicle event.
#[derive(Clone, Debug)]
pub struct ChronicleEvent {
    pub tick: u64,
    pub year: u32,
    pub text: String,
    pub importance: EventImportance,
}

/// Chronicle resource - stores the colony's historical record.
#[derive(Resource, Default)]
pub struct Chronicle {
    pub events: Vec<ChronicleEvent>,
}

impl Chronicle {
    /// Add a new event to the chronicle.
    pub fn add_event(&mut self, tick: u64, text: String, importance: EventImportance) {
        self.events.push(ChronicleEvent {
            tick,
            year: 1 + (tick / 1000) as u32, // Rough "year" approximation
            text,
            importance,
        });
    }
}

/// UI state for chronicle window.
#[derive(Resource, Default)]
pub struct ChronicleUiState {
    pub is_open: bool,
}

/// Tracks which building types have been built (for milestones).
#[derive(Resource, Default)]
pub struct BuildingTracker {
    pub has_built_housing: bool,
    pub has_built_farm: bool,
}
```

### Initial Event

```rust
// src/layer3/chronicle.rs

/// Creates the initial "colony founded" event.
pub fn initial_chronicle_event(world: &mut World) {
    world.resource_mut::<Chronicle>().add_event(
        0,
        "Colony founded. The journey begins.".to_string(),
        EventImportance::Legendary,
    );
}
```

### Milestone System

```rust
// src/layer3/chronicle.rs

use crate::layer2::building::{Building, BuildingType};
use crate::shared::time::SimulationTime;

/// Checks for building milestones and records them in the chronicle.
pub fn check_milestones_system(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;

    let buildings: Vec<BuildingType> = world
        .query::<&Building>()
        .iter(world)
        .map(|b| b.building_type)
        .collect();

    let mut tracker = world.resource_mut::<BuildingTracker>();
    let mut chronicle = world.resource_mut::<Chronicle>();

    for building_type in buildings {
        match building_type {
            BuildingType::Housing => {
                if !tracker.has_built_housing {
                    tracker.has_built_housing = true;
                    chronicle.add_event(
                        current_tick,
                        "First Housing constructed. A shelter from the void.".to_string(),
                        EventImportance::Major,
                    );
                }
            }
            BuildingType::Farm => {
                if !tracker.has_built_farm {
                    tracker.has_built_farm = true;
                    chronicle.add_event(
                        current_tick,
                        "First Farm operational. We shall not starve.".to_string(),
                        EventImportance::Major,
                    );
                }
            }
        }
    }
}
```

### Rendering Helper

```rust
// src/layer3/chronicle.rs

/// Format the importance prefix for display.
#[must_use]
pub fn format_event_prefix(importance: EventImportance) -> &'static str {
    match importance {
        EventImportance::Legendary => "!!!",
        EventImportance::Major => "!",
        _ => " ",
    }
}
```

### Input Handling

```rust
// src/main.rs - Update handle_input

use scale::layer3::ChronicleUiState;

fn handle_input(world: &mut World, key: crossterm::event::KeyEvent) {
    use crossterm::event::KeyCode;

    let build_active = world.resource::<BuildMode>().active;
    let chronicle_open = world.resource::<ChronicleUiState>().is_open;

    match key.code {
        // Chronicle toggle
        KeyCode::Char('l') | KeyCode::Char('h') | KeyCode::Char('L') | KeyCode::Char('H')
            if !build_active =>
        {
            let mut ui_state = world.resource_mut::<ChronicleUiState>();
            ui_state.is_open = !ui_state.is_open;

            // Auto-pause when opening
            if ui_state.is_open {
                *world.resource_mut::<GameState>() = GameState::Paused;
            }
        }

        // Close chronicle with Esc
        KeyCode::Esc if chronicle_open => {
            world.resource_mut::<ChronicleUiState>().is_open = false;
        }

        // ... rest of existing input handling ...
        // (make sure Esc for chronicle takes priority over Esc for build mode)
    }
}
```

### Rendering

```rust
// src/main.rs - Add render_chronicle function

use ratatui::widgets::{List, ListItem, Clear};
use scale::layer3::{Chronicle, ChronicleUiState, format_event_prefix};

fn render_chronicle(frame: &mut Frame, area: Rect, world: &World) {
    let ui_state = world.resource::<ChronicleUiState>();
    if !ui_state.is_open {
        return;
    }

    let chronicle = world.resource::<Chronicle>();

    // Helper to center rect
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        use ratatui::layout::{Constraint, Direction, Layout};

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

    let block = Block::default()
        .title(" Chronicle (Press L/H to close) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    let popup_area = centered_rect(60, 60, area);
    frame.render_widget(Clear, popup_area); // Clear background
    frame.render_widget(block.clone(), popup_area);

    let inner = block.inner(popup_area);

    let items: Vec<ListItem> = chronicle
        .events
        .iter()
        .rev() // Newest first
        .map(|evt| {
            let prefix = format_event_prefix(evt.importance);
            ListItem::new(format!(
                "[{}] Y{}: {} {}",
                evt.tick, evt.year, prefix, evt.text
            ))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

// In render() function, AFTER rendering map/info/status:
render_chronicle(frame, frame.area(), world);
```

### System Integration

```rust
// src/main.rs - In main()

use scale::layer3::{Chronicle, ChronicleUiState, BuildingTracker, initial_chronicle_event, check_milestones_system};

// After spawning initial pops:
world.insert_resource(Chronicle::default());
world.insert_resource(ChronicleUiState::default());
world.insert_resource(BuildingTracker::default());

initial_chronicle_event(&mut world);

// In simulation tick, AFTER job assignment:
check_milestones_system(&mut world);
```

### Module Integration

```rust
// src/layer3/mod.rs
pub mod chronicle;

pub use chronicle::{
    Chronicle, ChronicleEvent, EventImportance, ChronicleUiState, BuildingTracker,
    initial_chronicle_event, check_milestones_system, format_event_prefix,
};
```

```rust
// src/lib.rs
pub mod layer1;
pub mod layer2;
pub mod layer3;
pub mod shared;
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Year calculation is crude**: `tick / 1000` is arbitrary
   - Future: Define calendar system (ticks per day/month/year)
   - Current: Simple approximation for display

2. **No persistence**: Chronicle lost on exit
   - Future: Add save/load system
   - Current: Session-only acceptable for MVP

3. **No filtering/search**: Shows all events always
   - Future: Add importance filter, search box
   - Current: Simple list sufficient for first few hours of play

4. **Manual milestone tracking**: Each building type needs explicit code
   - Future: Define milestone config data, generic system
   - Current: Two building types, explicit is clear

5. **Pauses when opening**: No way to view while running
   - Future: Make pause optional or add transparency
   - Current: Pause prevents missing events

### Performance Considerations

- **Rendering is O(events)**: Linear in event count
- **Reverse iteration**: Creates new Vec (cheap for <1000 events)
- **Milestone check is O(buildings)**: Runs every tick but checks are fast

### API Design Notes

- `Chronicle` has public fields - acceptable for resources
- `add_event` calculates year automatically - consistent logic
- `ChronicleEvent` is cloneable - can be extracted for analysis
- Tracker booleans simple - more scalable would be HashSet<BuildingType>

### Future Extensibility

When adding more event types (future spec):
```rust
pub enum EventType {
    Foundation,
    BuildingMilestone(BuildingType),
    PopMilestone(usize), // Population reached X
    Disaster(DisasterType),
    Discovery,
}

impl Chronicle {
    pub fn add_typed_event(&mut self, tick: u64, event_type: EventType) {
        let (text, importance) = event_type.to_text_and_importance();
        self.add_event(tick, text, importance);
    }
}
```

When adding filtering:
```rust
pub struct ChronicleUiState {
    pub is_open: bool,
    pub min_importance: EventImportance,
    pub search_query: String,
}

// In rendering:
let filtered: Vec<_> = chronicle.events.iter()
    .filter(|e| e.importance >= ui_state.min_importance)
    .filter(|e| e.text.contains(&ui_state.search_query))
    .collect();
```

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for layer3/chronicle.rs
- [x] `Chronicle` resource exists
- [x] `L` or `H` key toggles the Chronicle window
- [x] Window is centered and covers part of map
- [x] Game auto-pauses when window opens
- [x] "Colony founded" event appears at start
- [x] Building a Farm triggers a milestone event (once only)
- [x] Building Housing triggers a milestone event (once only)
- [x] Events show tick/year and text
- [x] Events display in reverse chronological order (newest first)

## Technical Guidance

### Chronicle Window Layout

```
┌─────────────────────────────────────────────────┐
│              Chronicle Window                   │ 60% width
│  (Centered overlay)                             │ 60% height
│                                                 │
│  [1000] Y2: ! First Farm operational...         │
│  [500]  Y1: ! First Housing constructed...      │
│  [0]    Y1: !!! Colony founded...               │
│                                                 │
└─────────────────────────────────────────────────┘
```

- Uses `Clear` widget to hide content underneath
- Centered using layout constraints
- Black background for readability

### Event Prefix Indicators

| Importance | Prefix | Example |
|------------|--------|---------|
| Legendary | !!! | Colony founded |
| Major | ! | First building |
| Standard | (space) | Regular events |
| Minor | (space) | Small notices |

### Input Priority

With chronicle open:
1. Esc → Close chronicle (highest priority)
2. L/H → Toggle chronicle

With build mode active:
1. Esc → Exit build mode
2. Other keys → Build mode actions

**Chronicle takes precedence** - check `chronicle_open` before `build_active` in input handler.

### Common Pitfalls

1. **Forgetting initial event**: Chronicle starts empty without manual call
2. **Wrong render order**: Render chronicle LAST (on top of other UI)
3. **Not using Clear widget**: Background content shows through
4. **Wrong Esc priority**: Build mode Esc might conflict with chronicle Esc

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

## Congratulations!

With specs 001-010 complete, you have built a fully functional colony simulation game:

- **Terrain and movement** (001-002)
- **UI and time** (003)
- **Living colonists** (004-005)
- **Building construction** (006)
- **Survival systems** (007-008)
- **Autonomous job management** (009)
- **Historical storytelling** (010)

**The colony lives, works, and creates its own story.**

Next steps are open-ended: add more building types, implement disasters, create win conditions, or expand the simulation depth.
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
