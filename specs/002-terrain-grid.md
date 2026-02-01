# 002: Terrain Grid Generation

## Overview

Generate and render a 2D tile-based terrain map. This is the foundation of the colony—all buildings and pops exist on this grid.

## Dependencies

- `001` — Project scaffold must exist

## Requirements

### Must Have
- Grid size: 64x64 tiles (configurable constant)
- Tile size: 16x16 pixels
- Terrain types: `Grass`, `Dirt`, `Rock`, `Water`
- Simple procedural generation (noise-based or random clusters)
- Each tile rendered as a colored rectangle
- Grid stored as a resource for other systems to query

### Must NOT Have
- Underground layers (future spec)
- Resources on tiles (future spec)
- Tile selection or interaction

## Technical Guidance

### Components and Resources
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Grass,
    Dirt,
    Rock,
    Water,
}

impl TerrainType {
    pub fn color(&self) -> Color {
        match self {
            TerrainType::Grass => Color::srgb(0.2, 0.5, 0.2),
            TerrainType::Dirt => Color::srgb(0.4, 0.3, 0.2),
            TerrainType::Rock => Color::srgb(0.4, 0.4, 0.4),
            TerrainType::Water => Color::srgb(0.2, 0.3, 0.6),
        }
    }
}

#[derive(Resource)]
pub struct TerrainGrid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TerrainType>,  // row-major: index = y * width + x
}

impl TerrainGrid {
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainType> {
        if x < self.width && y < self.height {
            Some(self.tiles[y * self.width + x])
        } else {
            None
        }
    }
}

#[derive(Component)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
}
```

### Generation Approach

Simple approach without noise library:
1. Fill with Grass
2. Random walk to create Dirt patches
3. Random walk to create Rock clusters
4. Flood fill from edges for Water (or skip water for MVP)

### Rendering

Spawn one `Sprite` entity per tile with `Tile` component. Position calculated from grid coords.

```rust
let world_x = x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE / 2.0);
let world_y = y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE / 2.0);
```

Center the grid around world origin for camera sanity.

## Acceptance Criteria

- [ ] Running the game shows a 64x64 colored grid
- [ ] At least 3 terrain types visible (Grass, Dirt, Rock)
- [ ] Terrain has some variation (not uniform)
- [ ] Grid is centered in the window
- [ ] `TerrainGrid` resource is accessible
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
