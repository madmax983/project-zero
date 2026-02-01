# 004: Pop Spawning and Rendering

## Overview

Pops are the colonists—the individuals who work, eat, sleep, and die. This spec covers creating pop entities and rendering them on the map.

## Dependencies

- `001` — Project scaffold
- `002` — Terrain grid (pops exist on tiles)

## Requirements

### Must Have
- Pop component with grid position (x, y)
- Pops rendered as small colored circles on their tile
- Initial spawn: 5 pops at random walkable positions
- Pops appear above terrain (z-ordering)
- Pop position is grid-based (integer coords), not pixel-based

### Must NOT Have
- Movement (future spec)
- Needs like hunger (future spec)
- Selection or interaction
- Names or individual identity

## Technical Guidance

### Components

```rust
#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
}

impl GridPosition {
    pub fn to_world(&self) -> Vec2 {
        Vec2::new(
            self.x as f32 * TILE_SIZE - (GRID_WIDTH as f32 * TILE_SIZE / 2.0) + TILE_SIZE / 2.0,
            self.y as f32 * TILE_SIZE - (GRID_HEIGHT as f32 * TILE_SIZE / 2.0) + TILE_SIZE / 2.0,
        )
    }
}
```

### Spawning

```rust
fn spawn_initial_pops(
    mut commands: Commands,
    terrain: Res<TerrainGrid>,
) {
    let mut rng = rand::thread_rng();
    let mut spawned = 0;
    
    while spawned < 5 {
        let x = rng.gen_range(0..terrain.width);
        let y = rng.gen_range(0..terrain.height);
        
        // Only spawn on walkable terrain
        if let Some(terrain_type) = terrain.get(x, y) {
            if terrain_type != TerrainType::Water && terrain_type != TerrainType::Rock {
                let world_pos = GridPosition { x, y }.to_world();
                commands.spawn((
                    Pop,
                    GridPosition { x, y },
                    Sprite {
                        color: Color::srgb(0.9, 0.7, 0.5), // Flesh tone
                        custom_size: Some(Vec2::splat(TILE_SIZE * 0.6)),
                        ..default()
                    },
                    Transform::from_xyz(world_pos.x, world_pos.y, 1.0), // z=1 above terrain
                ));
                spawned += 1;
            }
        }
    }
}
```

### Dependency

Add `rand` to Cargo.toml:
```toml
rand = "0.8"
```

## Acceptance Criteria

- [ ] Running game shows 5 small circles on the terrain
- [ ] Pops only spawn on Grass or Dirt (not Water/Rock)
- [ ] Pops are visually above terrain tiles
- [ ] Pop positions are centered on their tiles
- [ ] Each pop has `Pop` and `GridPosition` components
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
