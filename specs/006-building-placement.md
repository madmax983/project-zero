# 006: Building Placement System

## Overview

Allow the player to place buildings using a cursor. Toggle build mode, move cursor, select building type, place on valid tiles. This establishes the player's primary interaction mechanic for colony construction.

## Dependencies

- `002` — Terrain grid
- `003` — UI layout

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer2/building.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_building_type_default() {
        let bt = BuildingType::default();
        assert_eq!(bt, BuildingType::Housing);
    }

    #[test]
    fn test_building_type_chars() {
        assert_eq!(BuildingType::Housing.char(), '⌂');
        assert_eq!(BuildingType::Farm.char(), '♣');
    }

    #[test]
    fn test_building_type_colors() {
        use ratatui::style::Color;
        assert_eq!(BuildingType::Housing.color(), Color::Rgb(139, 90, 43));
        assert_eq!(BuildingType::Farm.color(), Color::Rgb(218, 165, 32));
    }

    #[test]
    fn test_building_type_labels() {
        assert_eq!(BuildingType::Housing.label(), "Housing");
        assert_eq!(BuildingType::Farm.label(), "Farm");
    }

    #[test]
    fn test_building_type_next() {
        assert_eq!(BuildingType::Housing.next(), BuildingType::Farm);
        assert_eq!(BuildingType::Farm.next(), BuildingType::Housing);
    }

    #[test]
    fn test_building_component_creation() {
        let building = Building {
            building_type: BuildingType::Farm,
        };
        assert_eq!(building.building_type, BuildingType::Farm);
    }

    #[test]
    fn test_build_mode_default() {
        let mode = BuildMode::default();
        assert!(!mode.active);
        assert_eq!(mode.cursor.x, 0);
        assert_eq!(mode.cursor.y, 0);
        assert_eq!(mode.selected, BuildingType::Housing);
    }

    #[test]
    fn test_build_mode_toggle() {
        let mut mode = BuildMode::default();
        assert!(!mode.active);

        mode.active = true;
        assert!(mode.active);

        mode.active = !mode.active;
        assert!(!mode.active);
    }

    #[test]
    fn test_build_mode_cursor_movement() {
        let mut mode = BuildMode::default();
        mode.cursor.x = 5;
        mode.cursor.y = 10;

        mode.cursor.x += 1;
        mode.cursor.y -= 1;

        assert_eq!(mode.cursor.x, 6);
        assert_eq!(mode.cursor.y, 9);
    }

    #[test]
    fn test_build_mode_type_cycling() {
        let mut mode = BuildMode::default();
        assert_eq!(mode.selected, BuildingType::Housing);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Farm);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Housing);
    }

    #[test]
    fn test_occupied_tiles_default() {
        let occupied = OccupiedTiles::default();
        assert!(occupied.0.is_empty());
    }

    #[test]
    fn test_occupied_tiles_insertion() {
        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 10));
        assert!(occupied.0.contains(&(5, 10)));
        assert!(!occupied.0.contains(&(5, 11)));
    }

    #[test]
    fn test_can_place_on_grass() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let can_place = can_place_building(&world, 5, 5);
        assert!(can_place, "Should be able to place on grass");
    }

    #[test]
    fn test_cannot_place_on_water() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Water; // Position (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let can_place = can_place_building(&world, 5, 5);
        assert!(!can_place, "Should not be able to place on water");
    }

    #[test]
    fn test_cannot_place_on_rock() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // Position (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let can_place = can_place_building(&world, 5, 5);
        assert!(!can_place, "Should not be able to place on rock");
    }

    #[test]
    fn test_cannot_place_on_occupied() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        let can_place = can_place_building(&world, 5, 5);
        assert!(!can_place, "Should not be able to place on occupied tile");
    }

    #[test]
    fn test_cannot_place_out_of_bounds() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        assert!(!can_place_building(&world, -1, 5), "Negative x");
        assert!(!can_place_building(&world, 5, -1), "Negative y");
        assert!(!can_place_building(&world, 10, 5), "X out of bounds");
        assert!(!can_place_building(&world, 5, 10), "Y out of bounds");
    }

    #[test]
    fn test_place_building_success() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        try_place_building(&mut world, 5, 5, BuildingType::Farm);

        let count = world.query::<&Building>().iter(&world).count();
        assert_eq!(count, 1, "Should have spawned one building");

        let occupied = world.resource::<OccupiedTiles>();
        assert!(occupied.0.contains(&(5, 5)), "Tile should be marked occupied");
    }

    #[test]
    fn test_place_building_failure_water() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Water;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        try_place_building(&mut world, 5, 5, BuildingType::Farm);

        let count = world.query::<&Building>().iter(&world).count();
        assert_eq!(count, 0, "Should not spawn building on water");
    }

    #[test]
    fn test_building_has_position() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        try_place_building(&mut world, 7, 3, BuildingType::Housing);

        let (pos, _) = world.query::<(&GridPosition, &Building)>().single(&world);
        assert_eq!(pos.x, 7);
        assert_eq!(pos.y, 3);
    }
}
```

**Test Coverage Requirements:**
- BuildingType: all methods (char, color, label, next), all variants
- BuildMode: default state, toggle, cursor movement, type cycling
- OccupiedTiles: default empty, insertion, lookup
- can_place_building: valid terrain (Grass, Dirt), invalid (Water, Rock, occupied, out of bounds)
- try_place_building: successful placement, failed placement, tile marking
- Building spawning: has GridPosition component
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Components and Resources

```rust
// src/layer2/building.rs

use bevy_ecs::prelude::*;
use ratatui::style::Color;
use std::collections::HashSet;
use crate::layer1::GridPosition;

/// Building types available for construction.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BuildingType {
    #[default]
    Housing,
    Farm,
}

impl BuildingType {
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Housing => '⌂',
            Self::Farm => '♣',
        }
    }

    #[must_use]
    pub const fn color(&self) -> Color {
        match self {
            Self::Housing => Color::Rgb(139, 90, 43),  // Brown
            Self::Farm => Color::Rgb(218, 165, 32),    // Goldenrod
        }
    }

    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Housing => "Housing",
            Self::Farm => "Farm",
        }
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::Housing => Self::Farm,
            Self::Farm => Self::Housing,
        }
    }
}

/// Building component - attached to building entities.
#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
}

/// Build mode state resource.
#[derive(Resource, Default)]
pub struct BuildMode {
    pub active: bool,
    pub cursor: GridPosition,
    pub selected: BuildingType,
}

/// Tracks which tiles have buildings (for placement validation).
#[derive(Resource, Default)]
pub struct OccupiedTiles(pub HashSet<(i32, i32)>);
```

### Placement Logic

```rust
// src/layer2/building.rs

use crate::layer1::terrain::{TerrainGrid, TerrainType};

/// Check if a building can be placed at the given position.
#[must_use]
pub fn can_place_building(world: &World, x: i32, y: i32) -> bool {
    let terrain = world.resource::<TerrainGrid>();
    let occupied = world.resource::<OccupiedTiles>();

    // Check bounds
    if x < 0 || y < 0 {
        return false;
    }

    // Check terrain
    if let Some(tile) = terrain.get(x as usize, y as usize) {
        if tile == TerrainType::Water || tile == TerrainType::Rock {
            return false;
        }
    } else {
        return false; // Out of bounds
    }

    // Check occupation
    if occupied.0.contains(&(x, y)) {
        return false;
    }

    true
}

/// Attempt to place a building at the given position.
/// Returns true if successful, false if placement blocked.
pub fn try_place_building(
    world: &mut World,
    x: i32,
    y: i32,
    building_type: BuildingType,
) -> bool {
    if !can_place_building(world, x, y) {
        return false;
    }

    // Spawn building
    world.spawn((
        Building { building_type },
        GridPosition { x, y },
    ));

    // Mark tile occupied
    world.resource_mut::<OccupiedTiles>().0.insert((x, y));

    true
}
```

### Input Handling

```rust
// src/main.rs - Update handle_input function

use scale::layer2::{BuildMode, BuildingType};

fn handle_input(world: &mut World, key: crossterm::event::KeyEvent) {
    use crossterm::event::KeyCode;

    let build_active = world.resource::<BuildMode>().active;

    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc if !build_active => {
            *world.resource_mut::<GameState>() = GameState::Quitting;
        }

        // Pause
        KeyCode::Char(' ') if !build_active => {
            let mut state = world.resource_mut::<GameState>();
            *state = match *state {
                GameState::Running => GameState::Paused,
                GameState::Paused => GameState::Running,
                GameState::Quitting => GameState::Quitting,
            };
        }

        // Speed controls
        KeyCode::Char('1') if !build_active => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Normal;
        }
        KeyCode::Char('2') if !build_active => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Fast;
        }
        KeyCode::Char('3') if !build_active => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Faster;
        }

        // Build mode toggle
        KeyCode::Char('b') => {
            let mut build_mode = world.resource_mut::<BuildMode>();
            build_mode.active = !build_mode.active;
            if build_mode.active {
                // Initialize cursor to center of viewport
                let viewport = world.resource::<Viewport>();
                build_mode.cursor = GridPosition {
                    x: viewport.x + 10,
                    y: viewport.y + 10,
                };
            }
        }

        // Exit build mode
        KeyCode::Esc if build_active => {
            world.resource_mut::<BuildMode>().active = false;
        }

        // Cycle building type
        KeyCode::Tab if build_active => {
            let mut build_mode = world.resource_mut::<BuildMode>();
            build_mode.selected = build_mode.selected.next();
        }

        // Movement - cursor in build mode, viewport otherwise
        KeyCode::Up | KeyCode::Char('w') => {
            if build_active {
                world.resource_mut::<BuildMode>().cursor.y -= 1;
            } else {
                world.resource_mut::<Viewport>().y -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('s') => {
            if build_active {
                world.resource_mut::<BuildMode>().cursor.y += 1;
            } else {
                world.resource_mut::<Viewport>().y += 1;
            }
        }
        KeyCode::Left | KeyCode::Char('a') => {
            if build_active {
                world.resource_mut::<BuildMode>().cursor.x -= 1;
            } else {
                world.resource_mut::<Viewport>().x -= 1;
            }
        }
        KeyCode::Right | KeyCode::Char('d') => {
            if build_active {
                world.resource_mut::<BuildMode>().cursor.x += 1;
            } else {
                world.resource_mut::<Viewport>().x += 1;
            }
        }

        // Place building
        KeyCode::Enter if build_active => {
            let build_mode = world.resource::<BuildMode>();
            let cursor = build_mode.cursor;
            let building_type = build_mode.selected;
            try_place_building(world, cursor.x, cursor.y, building_type);
        }

        _ => {} // Ignore other keys
    }
}
```

### Rendering Updates

```rust
// src/layer1/terrain.rs - Update render_terrain_and_pops to render_map_layer

pub fn render_map_layer(
    frame: &mut Frame,
    area: Rect,
    terrain: &TerrainGrid,
    viewport: &Viewport,
    pops_data: &[(GridPosition, (char, Color))],
    buildings_data: &[(GridPosition, BuildingType)],
    build_mode: Option<(GridPosition, BuildingType, bool)>, // cursor, selected, can_place
) {
    let mut lines: Vec<Line> = Vec::new();

    for screen_y in 0..area.height {
        let world_y = viewport.y + screen_y as i32;
        let mut line_spans = Vec::new();

        for screen_x in 0..area.width {
            let world_x = viewport.x + screen_x as i32;

            // Build mode cursor (highest priority)
            if let Some((cursor, selected, can_place)) = build_mode {
                if cursor.x == world_x && cursor.y == world_y {
                    let bg = if can_place { Color::Green } else { Color::Red };
                    let ch = selected.char();
                    line_spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::White).bg(bg),
                    ));
                    continue;
                }
            }

            // Buildings
            if let Some((_, building_type)) = buildings_data.iter()
                .find(|(pos, _)| pos.x == world_x && pos.y == world_y)
            {
                line_spans.push(Span::styled(
                    building_type.char().to_string(),
                    Style::default().fg(building_type.color()),
                ));
                continue;
            }

            // Pops
            if let Some((_, (ch, color))) = pops_data.iter()
                .find(|(pos, _)| pos.x == world_x && pos.y == world_y)
            {
                line_spans.push(Span::styled(ch.to_string(), Style::default().fg(*color)));
                continue;
            }

            // Terrain (base layer)
            let (ch, color) = if world_x >= 0 && world_y >= 0 {
                if let Some(tile) = terrain.get(world_x as usize, world_y as usize) {
                    (tile.char(), tile.color())
                } else {
                    (' ', Color::Black)
                }
            } else {
                (' ', Color::Black)
            };

            line_spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        lines.push(Line::from(line_spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
```

```rust
// src/main.rs - Update render_map

fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();
    let build_mode = world.resource::<BuildMode>();

    // Collect pops
    let pops_data: Vec<(GridPosition, (char, Color))> = world
        .query::<(&GridPosition, &Needs)>()
        .iter(world)
        .map(|(pos, needs)| (*pos, pop_display(needs)))
        .collect();

    // Collect buildings
    let buildings_data: Vec<(GridPosition, BuildingType)> = world
        .query::<(&GridPosition, &Building)>()
        .iter(world)
        .map(|(pos, building)| (*pos, building.building_type))
        .collect();

    // Build mode cursor info
    let build_mode_cursor = if build_mode.active {
        let can_place = can_place_building(world, build_mode.cursor.x, build_mode.cursor.y);
        Some((build_mode.cursor, build_mode.selected, can_place))
    } else {
        None
    };

    render_map_layer(
        frame,
        inner,
        terrain,
        viewport,
        &pops_data,
        &buildings_data,
        build_mode_cursor,
    );
}
```

### Status Bar Update

```rust
// src/main.rs - Update render_status_bar

fn render_status_bar(frame: &mut Frame, area: Rect, world: &World) {
    let sim_time = world.resource::<SimulationTime>();
    let game_state = world.resource::<GameState>();
    let build_mode = world.resource::<BuildMode>();

    let paused = *game_state == GameState::Paused || sim_time.speed == SimSpeed::Paused;

    let mode_str = if build_mode.active {
        format!(
            "BUILD: {} (Tab:switch Enter:place Esc:exit)",
            build_mode.selected.label()
        )
    } else {
        "B:Build  1-3:Speed  q:Quit".to_string()
    };

    let status = format!(
        " {} │ Tick: {} │ {} │ {} ",
        if paused { "⏸" } else { "▶" },
        sim_time.tick,
        sim_time.speed.label(),
        mode_str,
    );

    let bar = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}
```

### Module Integration

```rust
// src/layer2/mod.rs
pub mod building;

pub use building::{
    Building, BuildingType, BuildMode, OccupiedTiles,
    can_place_building, try_place_building,
};
```

```rust
// src/main.rs - Add to imports

use scale::layer2::{BuildMode, OccupiedTiles, can_place_building, try_place_building};

// In main(), after spawn_initial_pops:
world.insert_resource(BuildMode::default());
world.insert_resource(OccupiedTiles::default());
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Cursor initialization hardcoded**: `viewport + 10` assumes screen size
   - Future: Calculate actual center based on terminal dimensions
   - Current: Works for standard terminal sizes (>20x20)

2. **Input routing complexity**: Many `if build_active` checks
   - Future: Create input context stack or state machine
   - Current: Explicit conditionals are readable

3. **can_place_building called twice**: Once for cursor highlight, once for placement
   - Future: Cache validation result in BuildMode
   - Current: Function is cheap (just lookups), premature optimization

4. **No undo**: Placed buildings are permanent
   - Future: Add demolish key or undo stack
   - Current: Out of scope for MVP

5. **Single-tile buildings only**: No multi-tile structures
   - Future: Add size field to BuildingType
   - Current: Simple 1x1 placement sufficient

### Performance Considerations

- **HashSet lookup**: O(1) for occupation check
- **Linear search for buildings**: O(n) where n = building count
  - Acceptable for <100 buildings
  - Future: Spatial index if needed
- **Rendering unchanged**: Still O(screen_area), building check is per-pixel

### API Design Notes

- `BuildingType` is Copy - cheap to pass around
- `try_place_building` returns bool - caller knows if placement succeeded
- `can_place_building` is pure - doesn't mutate world
- `OccupiedTiles` uses HashSet not Vec - O(1) lookups

### Future Extensibility

When adding building costs (future spec):
```rust
fn try_place_building(...) -> bool {
    if !can_place_building(...) {
        return false;
    }
    if !can_afford_building(world, building_type) {
        return false; // Not enough resources
    }
    // Spawn and deduct cost
    ...
}
```

When adding multi-tile buildings:
```rust
impl BuildingType {
    pub const fn size(&self) -> (usize, usize) {
        match self {
            Self::Housing => (1, 1),
            Self::Farm => (2, 2),
            Self::Warehouse => (3, 2),
        }
    }
}

fn can_place_building(world: &World, x: i32, y: i32, building_type: BuildingType) -> bool {
    let (w, h) = building_type.size();
    for dy in 0..h {
        for dx in 0..w {
            // Check each tile
        }
    }
}
```

When adding demolish:
```rust
fn try_demolish_building(world: &mut World, x: i32, y: i32) -> bool {
    // Find building at position
    // Despawn entity
    // Remove from OccupiedTiles
    // Optionally: refund resources
}
```

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for layer2/building.rs
- [x] B key toggles build mode
- [x] Cursor visible as highlighted cell in build mode
- [x] Cursor shows green on valid tiles, red on invalid
- [x] Arrow keys/WASD move cursor (not viewport) in build mode
- [x] Tab cycles between Housing and Farm
- [x] Enter places building if valid
- [x] Building character appears on tile after placement
- [x] Cannot place on Water, Rock, or existing building
- [x] Escape exits build mode
- [x] Status bar shows current mode and selection

## Technical Guidance

### Rendering Z-Order

From bottom to top:
1. Terrain (base)
2. Pops (entities)
3. Buildings (structures)
4. Cursor (UI overlay) - only in build mode

### Build Mode State Machine

```
Normal Mode:
  - B → Build Mode (initialize cursor)
  - WASD → Move viewport
  - Space → Pause
  - 1/2/3 → Speed

Build Mode:
  - Esc → Normal Mode
  - WASD → Move cursor
  - Tab → Cycle building type
  - Enter → Place building (if valid)
```

### Cursor Highlighting

- Green background: Placement valid
- Red background: Placement blocked
- White foreground: Always visible
- Shows selected building character (not terrain underneath)

### Common Pitfalls

1. **Forgetting to initialize BuildMode**: Must add to world resources
2. **Forgetting to initialize OccupiedTiles**: Placement will succeed but not track
3. **Wrong input routing**: Build mode keys still trigger normal mode actions
4. **Cursor out of viewport**: Cursor can move beyond visible area (intentional)
5. **Marking tile before spawn succeeds**: Always spawn first, then mark occupied

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
