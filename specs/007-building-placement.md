# 007: Building Placement System

## Overview

Allow the player to place buildings on the terrain. This is the core interaction loop—selecting a building type, clicking a tile, creating the structure.

## Dependencies

- `002` — Terrain grid
- `003` — Camera controls (need to click on world)

## Requirements

### Must Have
- Build mode toggle (B key)
- When in build mode, cursor shows ghost of building
- Left click places building on valid tile
- Right click or Escape exits build mode
- Buildings occupy one tile
- Cannot build on Water, Rock, or existing buildings
- Building component tracks what tile it's on

### Must NOT Have
- Multiple building types (this spec just does the placement system)
- Building costs/resources
- Building destruction

## Technical Guidance

### Resources and Components

```rust
#[derive(Resource, Default)]
pub struct BuildMode {
    pub active: bool,
    pub selected: Option<BuildingType>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    Housing,
    Farm,
    // Future: Mine, Factory, etc.
}

impl BuildingType {
    pub fn color(&self) -> Color {
        match self {
            BuildingType::Housing => Color::srgb(0.6, 0.4, 0.2), // Brown
            BuildingType::Farm => Color::srgb(0.8, 0.7, 0.2),    // Yellow
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            BuildingType::Housing => "Housing",
            BuildingType::Farm => "Farm",
        }
    }
}

#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
}

#[derive(Resource, Default)]
pub struct OccupiedTiles(pub HashSet<(usize, usize)>);
```

### Cursor to World Position

```rust
fn cursor_to_grid(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<(usize, usize)> {
    let cursor_pos = window.cursor_position()?;
    let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_pos)?;
    
    // Convert to grid coords
    let grid_x = ((world_pos.x + GRID_WIDTH as f32 * TILE_SIZE / 2.0) / TILE_SIZE) as i32;
    let grid_y = ((world_pos.y + GRID_HEIGHT as f32 * TILE_SIZE / 2.0) / TILE_SIZE) as i32;
    
    if grid_x >= 0 && grid_x < GRID_WIDTH as i32 && grid_y >= 0 && grid_y < GRID_HEIGHT as i32 {
        Some((grid_x as usize, grid_y as usize))
    } else {
        None
    }
}
```

### Ghost Preview

Spawn a semi-transparent sprite that follows the cursor when in build mode:

```rust
#[derive(Component)]
pub struct BuildGhost;

fn update_build_ghost(
    build_mode: Res<BuildMode>,
    // ... cursor position logic
    mut ghost_query: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<BuildGhost>>,
) {
    // Move ghost to cursor grid position
    // Tint red if invalid placement, normal if valid
}
```

### Placement Validation

```rust
fn can_place_building(
    x: usize,
    y: usize,
    terrain: &TerrainGrid,
    occupied: &OccupiedTiles,
) -> bool {
    match terrain.get(x, y) {
        Some(TerrainType::Grass) | Some(TerrainType::Dirt) => {
            !occupied.0.contains(&(x, y))
        }
        _ => false,
    }
}
```

### Input Handling

```rust
fn handle_build_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut build_mode: ResMut<BuildMode>,
    mut commands: Commands,
    // ... other resources
) {
    // B toggles build mode
    if keyboard.just_pressed(KeyCode::KeyB) {
        build_mode.active = !build_mode.active;
        if build_mode.active {
            build_mode.selected = Some(BuildingType::Housing); // Default
        }
    }
    
    // Escape exits build mode
    if keyboard.just_pressed(KeyCode::Escape) {
        build_mode.active = false;
    }
    
    // Left click places
    if build_mode.active && mouse.just_pressed(MouseButton::Left) {
        // Validate and spawn building
    }
}
```

## Acceptance Criteria

- [ ] B key toggles build mode
- [ ] Ghost preview appears at cursor in build mode
- [ ] Ghost is red on invalid tiles, normal on valid
- [ ] Left click places building
- [ ] Building sprite appears on tile
- [ ] Cannot place on Water, Rock, or existing building
- [ ] Escape exits build mode
- [ ] `OccupiedTiles` resource tracks placed buildings
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
