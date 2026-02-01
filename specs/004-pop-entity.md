# 004: Pop Spawning and Rendering

## Overview

Pops are the colonists—individuals who work, eat, sleep, and die. This spec covers creating pop entities and rendering them on the map as characters.

## Dependencies

- `001` — Project scaffold
- `002` — Terrain grid (pops exist on tiles)
- `003` — UI layout (rendering context)

## Requirements

### Must Have
- Pop component with grid position (x, y)
- Pops rendered as `☺` character on their tile
- Pops render in a distinct color (yellow/tan)
- Initial spawn: 5 pops at random walkable positions
- Pops render on top of terrain

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

#[derive(Component, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}
```

### Spawning

```rust
pub fn spawn_initial_pops(world: &mut World) {
    let terrain = world.resource::<TerrainGrid>();
    let mut rng = rand::thread_rng();
    let mut spawned = 0;
    
    while spawned < 5 {
        let x = rng.gen_range(0..terrain.width as i32);
        let y = rng.gen_range(0..terrain.height as i32);
        
        // Only spawn on walkable terrain
        if let Some(terrain_type) = terrain.get(x as usize, y as usize) {
            if terrain_type != TerrainType::Water && terrain_type != TerrainType::Rock {
                world.spawn((
                    Pop,
                    GridPosition { x, y },
                ));
                spawned += 1;
            }
        }
    }
}
```

### Rendering

Modify the terrain rendering to overlay pops:

```rust
fn render_terrain_and_pops(
    frame: &mut Frame, 
    area: Rect, 
    terrain: &TerrainGrid, 
    viewport: &Viewport,
    pops: &[(GridPosition, /* later: other data */)],
) {
    // Build a lookup of pop positions
    let pop_positions: HashSet<(i32, i32)> = pops.iter()
        .map(|(pos, ..)| (pos.x, pos.y))
        .collect();
    
    let mut lines: Vec<Line> = Vec::new();
    
    for screen_y in 0..area.height {
        let world_y = viewport.y + screen_y as i32;
        let mut spans = Vec::new();
        
        for screen_x in 0..area.width {
            let world_x = viewport.x + screen_x as i32;
            
            // Check for pop first
            if pop_positions.contains(&(world_x, world_y)) {
                spans.push(Span::styled("☺", Style::default().fg(Color::Yellow)));
                continue;
            }
            
            // Otherwise render terrain
            let (ch, color) = if world_x >= 0 && world_y >= 0 {
                if let Some(tile) = terrain.get(world_x as usize, world_y as usize) {
                    (tile.char(), tile.color())
                } else {
                    (' ', Color::Black)
                }
            } else {
                (' ', Color::Black)
            };
            
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        lines.push(Line::from(spans));
    }
    
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
```

### Querying Pops for Rendering

In the render function, query all pops:

```rust
fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();
    
    // Collect pop data
    let mut pops_data = Vec::new();
    let mut query_state = world.query::<&GridPosition>().with::<Pop>();
    for pos in query_state.iter(world) {
        pops_data.push((*pos,));
    }
    
    render_terrain_and_pops(frame, inner, terrain, viewport, &pops_data);
}
```

## Acceptance Criteria

- [ ] Running game shows 5 `☺` characters on the terrain
- [ ] Pops only spawn on Grass or Dirt (not Water/Rock)
- [ ] Pops render in yellow/visible color
- [ ] Pops stay visible when scrolling (if in viewport)
- [ ] Each pop has `Pop` and `GridPosition` components
- [ ] Pops appear on top of terrain characters
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
