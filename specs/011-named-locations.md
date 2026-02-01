# 011: Named Locations

## Overview

The world should feel lived-in. Significant locations on the map acquire names based on history. These names persist and are displayed to the player, grounding the abstract grid in narrative.

## Dependencies

- `002` — Terrain grid (coordinates)
- `003` — UI layout (status bar display)
- `010` — Chronicle system (logging naming events)

## Requirements

### Must Have
- `NamedLocations` resource mapping coordinates to strings
- "Landing Site" automatically named at colony start location (pop spawn point)
- UI Status Bar shows the location name when the focused tile has a name
    - Focus = Build Mode Cursor (if active) OR Viewport Center (if inactive)
- Naming a location triggers a `LOCATION_NAMED` chronicle event
- Support for unique names (prevent duplicates if possible, or just string storage)

### Must NOT Have
- Player ability to manually rename locations (for now)
- Procedural name generation (use hardcoded strings for MVP events)
- Floating text on map (UI display only)

## Technical Guidance

### Data Structures

```rust
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct NamedLocations {
    // Map (x, y) -> Name
    pub names: HashMap<(i32, i32), String>,
}

impl NamedLocations {
    pub fn add(&mut self, x: i32, y: i32, name: String) {
        self.names.insert((x, y), name);
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&String> {
        self.names.get(&(x, y))
    }
}
```

### Initialization

In a startup system (after terrain/pops), name the center:

```rust
fn initial_naming_system(
    mut locations: ResMut<NamedLocations>,
    mut chronicle: ResMut<Chronicle>,
    viewport: Res<Viewport>,
    time: Res<SimulationTime>,
) {
    // Assuming viewport starts focused on the colony
    // Adjust x/y to match where pops actually spawned if different
    let x = viewport.x + 10; // Approx center if viewport is top-left
    let y = viewport.y + 10;

    let name = "Landing Site".to_string();

    locations.add(x, y, name.clone());

    // Note: Template ID is LOCATION_NAMED, but for MVP just adding text is fine
    // or use a structured event if Chronicle supports it.
    chronicle.add_event(
        time.tick,
        format!("We name this place {}. Here we begin.", name),
        EventImportance::Standard
    );
}
```

### UI Integration

Update the status bar system to check for named locations.

```rust
fn render_status_bar(
    // ... existing args
    locations: Res<NamedLocations>,
    viewport: Res<Viewport>,
    build_mode: Res<BuildMode>,
    // area: Rect (passed from parent or derived)
) {
    // Determine "focused" tile
    let (focus_x, focus_y) = if build_mode.active {
        (build_mode.cursor.x, build_mode.cursor.y)
    } else {
        // Calculate center of viewport
        // Assuming viewport size is roughly known or passed in render context
        // For MVP, just using Viewport.x + offset is fine
        (viewport.x + 10, viewport.y + 10)
    };

    let location_text = if let Some(name) = locations.get(focus_x, focus_y) {
        format!(" 📍 {} ", name)
    } else {
        String::new()
    };

    // Append to status string
    let status = format!(
        // ... existing status
        "{}{}",
        // ...
        location_text
    );
}
```

## Acceptance Criteria

- [ ] `NamedLocations` resource exists
- [ ] Game start creates "Landing Site" at initial coordinates
- [ ] Status bar displays "Landing Site" when focused on that tile
- [ ] Chronicle records the naming event
- [ ] Moving away from the tile hides the name
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
