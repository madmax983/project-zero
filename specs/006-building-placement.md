# 006: Building Placement System

## Overview

Allow the player to place buildings using a cursor. Toggle build mode, move cursor, select building type, place on valid tiles.

## Dependencies

- `002` — Terrain grid
- `003` — UI layout

## Requirements

### Must Have
- Build mode toggle (B key)
- Cursor rendered as blinking or highlighted cell
- Cursor movement with arrow keys (separate from viewport scroll in build mode)
- Building type selection (Tab to cycle, or number keys)
- Enter/Space places building on valid tile
- Escape exits build mode
- Cannot build on Water, Rock, or existing buildings
- Status bar shows current mode and selected building

### Must NOT Have
- Building costs/resources
- Building destruction
- Actual building functionality (just placement)

## Technical Guidance

### Resources and Components

```rust
#[derive(Resource, Default)]
pub struct BuildMode {
    pub active: bool,
    pub cursor: GridPosition,
    pub selected: BuildingType,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildingType {
    #[default]
    Housing,
    Farm,
}

impl BuildingType {
    pub fn char(&self) -> char {
        match self {
            BuildingType::Housing => '⌂',
            BuildingType::Farm => '♣',
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            BuildingType::Housing => Color::Rgb(139, 90, 43),  // Brown
            BuildingType::Farm => Color::Rgb(218, 165, 32),    // Goldenrod
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            BuildingType::Housing => "Housing",
            BuildingType::Farm => "Farm",
        }
    }
    
    pub fn next(&self) -> Self {
        match self {
            BuildingType::Housing => BuildingType::Farm,
            BuildingType::Farm => BuildingType::Housing,
        }
    }
}

#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
}

#[derive(Resource, Default)]
pub struct OccupiedTiles(pub HashSet<(i32, i32)>);
```

### Input Handling

```rust
fn handle_input(world: &mut World, key: KeyEvent) {
    let build_active = world.resource::<BuildMode>().active;
    
    match key.code {
        KeyCode::Char('b') => {
            let mut build_mode = world.resource_mut::<BuildMode>();
            build_mode.active = !build_mode.active;
            if build_mode.active {
                // Initialize cursor to center of viewport
                let viewport = world.resource::<Viewport>();
                build_mode.cursor = GridPosition { 
                    x: viewport.x + 10, 
                    y: viewport.y + 10 
                };
            }
        }
        KeyCode::Esc if build_active => {
            world.resource_mut::<BuildMode>().active = false;
        }
        KeyCode::Tab if build_active => {
            let mut build_mode = world.resource_mut::<BuildMode>();
            build_mode.selected = build_mode.selected.next();
        }
        // Arrow keys move cursor in build mode, viewport otherwise
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
        KeyCode::Enter | KeyCode::Char(' ') if build_active => {
            try_place_building(world);
        }
        // ... other input handling
    }
}
```

### Placement Logic

```rust
fn try_place_building(world: &mut World) {
    let build_mode = world.resource::<BuildMode>();
    let cursor = build_mode.cursor;
    let building_type = build_mode.selected;
    
    let terrain = world.resource::<TerrainGrid>();
    let occupied = world.resource::<OccupiedTiles>();
    
    // Validate placement
    let can_place = if cursor.x < 0 || cursor.y < 0 {
        false
    } else if let Some(tile) = terrain.get(cursor.x as usize, cursor.y as usize) {
        tile != TerrainType::Water && 
        tile != TerrainType::Rock && 
        !occupied.0.contains(&(cursor.x, cursor.y))
    } else {
        false
    };
    
    if can_place {
        // Spawn building
        world.spawn((
            Building { building_type },
            GridPosition { x: cursor.x, y: cursor.y },
        ));
        
        // Mark tile occupied
        world.resource_mut::<OccupiedTiles>().0.insert((cursor.x, cursor.y));
    }
}
```

### Rendering Cursor and Buildings

```rust
fn render_terrain_and_entities(/* ... */) {
    let build_mode = world.resource::<BuildMode>();
    let buildings: Vec<_> = world.query::<(&GridPosition, &Building)>()
        .iter(world)
        .map(|(p, b)| (*p, b.building_type))
        .collect();
    
    for screen_y in 0..area.height {
        let world_y = viewport.y + screen_y as i32;
        let mut spans = Vec::new();
        
        for screen_x in 0..area.width {
            let world_x = viewport.x + screen_x as i32;
            
            // Build mode cursor (highlight)
            if build_mode.active && 
               build_mode.cursor.x == world_x && 
               build_mode.cursor.y == world_y 
            {
                let can_place = /* validation check */;
                let bg = if can_place { Color::Green } else { Color::Red };
                let ch = build_mode.selected.char();
                spans.push(Span::styled(
                    ch.to_string(), 
                    Style::default().fg(Color::White).bg(bg)
                ));
                continue;
            }
            
            // Buildings
            if let Some((_, bt)) = buildings.iter().find(|(p, _)| p.x == world_x && p.y == world_y) {
                spans.push(Span::styled(
                    bt.char().to_string(),
                    Style::default().fg(bt.color())
                ));
                continue;
            }
            
            // Pops... (existing code)
            
            // Terrain... (existing code)
        }
        lines.push(Line::from(spans));
    }
}
```

### Status Bar Update

Show build mode state:

```rust
fn render_status_bar(/* ... */) {
    let build_mode = world.resource::<BuildMode>();
    
    let mode_str = if build_mode.active {
        format!("BUILD: {} (Tab:switch Enter:place Esc:exit)", build_mode.selected.label())
    } else {
        "B:Build".to_string()
    };
    
    let status = format!(
        " {} │ Tick: {} │ {} │ {} ",
        if sim_time.speed == SimSpeed::Paused { "⏸" } else { "▶" },
        sim_time.tick,
        sim_time.speed.label(),
        mode_str,
    );
    // ...
}
```

## Acceptance Criteria

- [ ] B key toggles build mode
- [ ] Cursor visible as highlighted cell in build mode
- [ ] Cursor shows green on valid tiles, red on invalid
- [ ] Arrow keys move cursor (not viewport) in build mode
- [ ] Tab cycles between Housing and Farm
- [ ] Enter/Space places building
- [ ] Building character appears on tile after placement
- [ ] Cannot place on Water, Rock, or existing building
- [ ] Escape exits build mode
- [ ] Status bar shows current mode and selection
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
