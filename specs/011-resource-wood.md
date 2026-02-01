# 011: Trees and Basic Resources

## Overview

Introduces natural resources (Wood, Stone) to the economy and adds Trees as interactive entities on the map. This is the foundation for construction costs and the gathering loop.

## Dependencies

- `008` — Farm/Food (defines `ColonyResources`)
- `002` — Terrain Grid (for spawning trees)

## Requirements

### Must Have
- `ColonyResources` tracks `wood` and `stone` (f32)
- `Tree` entity that exists on specific tiles
- Trees spawn on Grass or Dirt during generation
- Trees render as `↑` (Green)
- Trees block building placement (like existing buildings)
- Trees do NOT block movement (yet)

### Must NOT Have
- Gathering/Chopping (Spec 013)
- Tree growth/planting
- Stone items (Stone comes from existing Rock terrain)

## Technical Guidance

### Resources

Update `ColonyResources`:

```rust
#[derive(Resource, Default)]
pub struct ColonyResources {
    pub food: f32,
    pub wood: f32,
    pub stone: f32,
}
```

### Components

```rust
#[derive(Component)]
pub struct Tree;
```

### Tree Generation

Create a new system or function `spawn_trees` called during startup/generation.

```rust
pub fn spawn_trees(world: &mut World) {
    let terrain = world.resource::<TerrainGrid>();
    let mut rng = rand::thread_rng();

    // Attempt to spawn trees on valid terrain
    // Density: roughly 10% of map
    let target_trees = (terrain.width * terrain.height) / 10;

    for _ in 0..target_trees {
        let x = rng.gen_range(0..terrain.width as i32);
        let y = rng.gen_range(0..terrain.height as i32);

        // Check terrain type
        if let Some(tile) = terrain.get(x as usize, y as usize) {
            if matches!(tile, TerrainType::Grass | TerrainType::Dirt) {
                // Check if occupied (simplest: just spawn, collision check later)
                // Better: query existing entities at x,y

                world.spawn((
                    Tree,
                    GridPosition { x, y },
                ));
            }
        }
    }
}
```

*Note: You might want to consolidate "OccupiedTiles" logic if not already robust, but for now simple random spawn is fine.*

### Rendering

Update map rendering loop to include Trees.

```rust
// In render loop
if let Some(_) = trees.iter().find(|(p, _)| p.x == world_x && p.y == world_y) {
    spans.push(Span::styled("↑", Style::default().fg(Color::Green)));
    continue;
}
```

### Info Panel

Update Info Panel to show Wood/Stone counts.

## Acceptance Criteria

- [ ] Map contains `↑` characters (Trees) scattered on Grass/Dirt
- [ ] Trees do not spawn on Water or Rock
- [ ] `ColonyResources` has `wood` and `stone` fields
- [ ] Info Panel displays Wood: 0.0 and Stone: 0.0
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
