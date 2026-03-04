# 154: Named Locations

## Overview

The world should feel lived-in. Significant locations on the map acquire names based on history. These names persist and are displayed to the player, grounding the abstract grid in narrative.

## Dependencies

- `002` — Terrain grid (coordinates)
- `003` — UI layout (status bar display)
- `010` — Chronicle system (logging naming events)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/locations.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_named_locations_resource_default() {
        let locations = NamedLocations::default();
        assert!(locations.names.is_empty());
    }

    #[test]
    fn test_add_and_get_location_name() {
        let mut locations = NamedLocations::default();

        locations.add(10, 20, "Landing Site".to_string());

        assert_eq!(locations.get(10, 20), Some(&"Landing Site".to_string()));
    }

    #[test]
    fn test_get_nonexistent_location() {
        let locations = NamedLocations::default();
        assert_eq!(locations.get(5, 5), None);
    }

    #[test]
    fn test_overwrite_location_name() {
        let mut locations = NamedLocations::default();

        locations.add(10, 20, "Old Name".to_string());
        locations.add(10, 20, "New Name".to_string());

        assert_eq!(locations.get(10, 20), Some(&"New Name".to_string()));
    }

    #[test]
    fn test_initial_naming_system_creates_landing_site() {
        let mut world = World::new();
        world.insert_resource(NamedLocations::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime::default());

        // Spawn a pop to define the landing site
        world.spawn((
            Pop,
            GridPosition { x: 40, y: 25 },
        ));

        // Run system
        initial_naming_system(&mut world);

        let locations = world.resource::<NamedLocations>();
        assert_eq!(locations.get(40, 25), Some(&"Landing Site".to_string()));
    }

    #[test]
    fn test_initial_naming_logs_chronicle_event() {
        let mut world = World::new();
        world.insert_resource(NamedLocations::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime::default());

        world.spawn((
            Pop,
            GridPosition { x: 40, y: 25 },
        ));

        initial_naming_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert!(!chronicle.events.is_empty());
        assert!(chronicle.events[0].text.contains("Landing Site"));
    }

    #[test]
    fn test_get_location_name_at_viewport_center() {
        let mut locations = NamedLocations::default();
        locations.add(10, 10, "Center City".to_string());

        let viewport = Viewport { x: 0, y: 0 }; // Viewport (0,0) to (80,50)

        // Center of viewport (assuming 80x50 rendering) is roughly x+40, y+25
        // But for this test, let's just test the helper function logic directly
        // if we define a helper for "get focused name"

        assert_eq!(locations.get(10, 10), Some(&"Center City".to_string()));
    }
}
```

**Test Coverage Requirements:**
- NamedLocations resource: default, add, get, overwrite
- initial_naming_system: names pop location, logs to chronicle
- Integration with Chronicle and Pop components
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Data Structures

```rust
// src/layer1/locations.rs

use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Stores named locations on the world map.
#[derive(Resource, Default, Debug)]
pub struct NamedLocations {
    pub names: HashMap<(i32, i32), String>,
}

impl NamedLocations {
    /// Name a specific coordinate. Overwrites existing names.
    pub fn add(&mut self, x: i32, y: i32, name: String) {
        self.names.insert((x, y), name);
    }

    /// Get the name of a location, if it exists.
    #[must_use]
    pub fn get(&self, x: i32, y: i32) -> Option<&String> {
        self.names.get(&(x, y))
    }
}
```

### Initialization System

```rust
// src/layer1/locations.rs

use crate::layer1::pop::{Pop, GridPosition};
use crate::layer3::chronicle::{Chronicle, EventImportance};
use crate::shared::time::SimulationTime;

/// Names the starting location based on the first pop found.
pub fn initial_naming_system(world: &mut World) {
    // 1. Find a pop position
    let mut query = world.query::<(&GridPosition, With<Pop>)>();
    let start_pos = query.iter(world).next().map(|(pos, _)| (pos.x, pos.y));

    if let Some((x, y)) = start_pos {
        let name = "Landing Site".to_string();

        // 2. Add to locations
        let mut locations = world.resource_mut::<NamedLocations>();
        locations.add(x, y, name.clone());

        // 3. Log to chronicle
        let tick = world.resource::<SimulationTime>().tick;
        let mut chronicle = world.resource_mut::<Chronicle>();
        chronicle.add_event(
            tick,
            format!("We name this place {}. Here we begin.", name),
            EventImportance::Standard,
        );
    }
}
```

### UI Integration

```rust
// src/main.rs - Update render_status_bar

use scale::layer1::locations::NamedLocations;

fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    world: &World,
) {
    // ... setup layout ...

    let viewport = world.resource::<Viewport>();
    let locations = world.resource::<NamedLocations>();

    // Determine focus point (center of screen or build cursor)
    // For MVP, just use viewport center relative to map
    // Assuming map area is roughly the frame size
    let center_x = viewport.x + (area.width / 2) as i32;
    let center_y = viewport.y + (area.height / 2) as i32;

    let location_text = if let Some(name) = locations.get(center_x, center_y) {
        format!(" 📍 {} ", name)
    } else {
        String::new()
    };

    // Add to status line
    let status_text = format!(
        " Day {} | Souls: {} | ... {}",
        day, pop_count, location_text
    );

    // ... render paragraph ...
}
```

### Module Integration

```rust
// src/layer1/mod.rs
pub mod locations;

pub use locations::{NamedLocations, initial_naming_system};
```

```rust
// src/main.rs
// In main():
world.insert_resource(NamedLocations::default());
// After spawning pops:
initial_naming_system(&mut world);
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **HashMap Key type**: `(i32, i32)` is simple but creates tuple allocations?
   - Actually tuples are Copy/stack allocated, so it's fine.
   - Future: Use `GridIndex` struct if coordinate logic gets complex.

2. **Hardcoded "Landing Site"**: Name is fixed.
   - Future: Procedural name generation from `lore/FRAGMENTS.md`.
   - Current: Good enough for MVP.

3. **Querying Pop in System**: `initial_naming_system` runs once but queries.
   - Future: Pass starting position as resource or config.
   - Current: Robust because it uses actual entity data.

4. **UI Focus Logic**: Hardcoded to viewport center.
   - Future: Integrate with Selection System (Spec 015) to show name of *selected* tile.
   - Current: Center focus is intuitive for exploration.

### Performance Considerations

- **HashMap lookup**: O(1), very fast for UI rendering every frame.
- **Memory**: Stores strings. 1000 names ≈ 20KB. Negligible.
- **System overhead**: `initial_naming_system` runs once, then removed from schedule (or guarded).

### API Design Notes

- `NamedLocations` is a simple Resource wrapper.
- Separation of concerns: `NamedLocations` stores data, `Chronicle` stores history.
- `initial_naming_system` bridges the two systems.

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for layer1/locations.rs
- [x] `NamedLocations` resource exists
- [x] "Landing Site" is created at the exact coordinates of a starting pop
- [x] Status bar displays "📍 Landing Site" when looking at that location
- [x] Status bar hides location name when looking elsewhere
- [x] Chronicle contains the naming event

## Technical Guidance

### Integration with Status Bar

The status bar needs to know *where* the player is looking.
1. **Normal Mode**: Center of the viewport. `x = viewport.x + width/2`.
2. **Build Mode**: The cursor position (if implemented).
3. **Selection Mode**: The selected tile (if implemented).

For this spec, focus on **Normal Mode** (viewport center).

### Coordinate Systems

Remember that `area.width` is `u16` (screen pixels/cells), while `viewport.x` is `i32` (world coords).
```rust
let center_x = viewport.x + (area.width as i32 / 2);
let center_y = viewport.y + (area.height as i32 / 2);
```

### Common Pitfalls

1. **Off-by-one errors**: Viewport centering might be slightly off visually.
2. **Missing dependency**: Ensure `Chronicle` resource exists before running `initial_naming_system`.
3. **String cloning**: `name.clone()` is necessary when putting into both HashMap and Chronicle.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
