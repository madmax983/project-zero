# 092: Antagonistic Flora

## Overview

Introduces hostile plant life (e.g., **XenoMoss**, **StrangleVines**) that actively spreads across the map and damages man-made structures.
Unlike static trees (`019`), these are entities that grow, spread to adjacent tiles, and attack buildings.
Players must actively "Clear" them to protect their colony.

## Dependencies

- `045` — Structure Durability (Target for damage)
- `017` — Designation System (Interaction)
- `002` — Terrain Grid (Spawn location)

## RED Phase: Tests First

Write these tests in `src/layer1/flora_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_app::App;
    use crate::layer1::flora::{Flora, FloraType, flora_spread_system, flora_attack_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    fn setup_app() -> App {
        let mut app = App::new();
        let tiles = vec![TerrainType::Grass; 100]; // 10x10 grid
        app.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        app.insert_resource(crate::shared::time::SimulationTime::default());
        app
    }

    #[test]
    fn test_flora_component_defaults() {
        let flora = Flora::default();
        assert_eq!(flora.flora_type, FloraType::XenoMoss);
        assert!(flora.growth_timer > 0);
        assert!(flora.spread_chance > 0.0);
    }

    #[test]
    fn test_flora_spreads_to_adjacent_tile() {
        let mut app = setup_app();
        app.add_systems(Update, flora_spread_system);

        // Spawn Flora at (5, 5)
        app.world_mut().spawn((
            Flora {
                growth_timer: 0, // Ready to spread
                spread_chance: 1.0, // Guaranteed
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system
        app.update();

        // Check for new Flora entities
        let count = app.world().query::<&Flora>().iter(app.world()).count();
        assert!(count > 1, "Flora should have spread");

        // Verify new position is adjacent
        let mut positions = app.world().query::<(&Flora, &GridPosition)>().iter(app.world());
        let (_, p1) = positions.next().unwrap();
        let (_, p2) = positions.next().unwrap();

        let dx = (p1.x - p2.x).abs();
        let dy = (p1.y - p2.y).abs();
        assert!(dx <= 1 && dy <= 1 && (dx + dy) > 0, "New flora should be adjacent");
    }

    #[test]
    fn test_flora_does_not_spread_on_occupied_tile() {
        let mut app = setup_app();
        app.add_systems(Update, flora_spread_system);

        // Surround (5,5) with existing Flora
        for x in 4..=6 {
            for y in 4..=6 {
                if x == 5 && y == 5 { continue; }
                app.world_mut().spawn((
                    Flora::default(),
                    GridPosition { x, y },
                ));
            }
        }

        // Spawn central Flora
        app.world_mut().spawn((
            Flora { growth_timer: 0, spread_chance: 1.0, ..Default::default() },
            GridPosition { x: 5, y: 5 },
        ));

        let initial_count = app.world().entities().len();

        app.update();

        assert_eq!(app.world().entities().len(), initial_count, "Should not spread to occupied tiles");
    }

    #[test]
    fn test_flora_damages_building_on_same_tile() {
        let mut app = setup_app();
        app.add_systems(Update, flora_attack_system);

        // Spawn Building
        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            Structure { current_hp: 100.0, max_hp: 100.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn Flora on same tile (Attack mode)
        app.world_mut().spawn((
            Flora {
                damage: 10.0,
                attack_timer: 0,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        app.update();

        let structure = app.world().get::<Structure>(building).unwrap();
        assert_eq!(structure.current_hp, 90.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Flora` Component

In `src/layer1/flora.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloraType {
    #[default]
    XenoMoss,
    StrangleVines,
}

#[derive(Component)]
pub struct Flora {
    pub flora_type: FloraType,
    pub growth_timer: u32,
    pub spread_chance: f32,
    pub attack_timer: u32,
    pub damage: f32,
}

impl Default for Flora {
    fn default() -> Self {
        Self {
            flora_type: FloraType::XenoMoss,
            growth_timer: 100,
            spread_chance: 0.1,
            attack_timer: 50,
            damage: 5.0,
        }
    }
}
```

### 2. Implement `flora_spread_system`

```rust
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainGrid;
use rand::Rng;

pub fn flora_spread_system(mut commands: Commands, mut query: Query<(&mut Flora, &GridPosition)>, terrain: Res<TerrainGrid>, other_flora: Query<&GridPosition, With<Flora>>) {
    let mut rng = rand::thread_rng();

    // Cache occupied positions for speed
    let occupied: std::collections::HashSet<(i32, i32)> = other_flora.iter().map(|p| (p.x, p.y)).collect();

    for (mut flora, pos) in &mut query {
        if flora.growth_timer > 0 {
            flora.growth_timer -= 1;
            continue;
        }

        // Reset timer
        flora.growth_timer = 100;

        if rng.gen::<f32>() > flora.spread_chance {
            continue;
        }

        // Pick random adjacent
        let dx = rng.gen_range(-1..=1);
        let dy = rng.gen_range(-1..=1);
        if dx == 0 && dy == 0 { continue; }

        let nx = pos.x + dx;
        let ny = pos.y + dy;

        // Check bounds and occupancy
        if nx >= 0 && nx < terrain.width as i32 && ny >= 0 && ny < terrain.height as i32 {
            if !occupied.contains(&(nx, ny)) {
                // Spawn new
                commands.spawn((
                    Flora::default(),
                    GridPosition { x: nx, y: ny },
                ));
            }
        }
    }
}
```

### 3. Implement `flora_attack_system`

```rust
use crate::layer1::structure::Structure;

pub fn flora_attack_system(mut flora_query: Query<(&mut Flora, &GridPosition)>, mut buildings: Query<(&mut Structure, &GridPosition)>) {
    for (mut flora, flora_pos) in &mut flora_query {
        if flora.attack_timer > 0 {
            flora.attack_timer -= 1;
            continue;
        }

        flora.attack_timer = 50;

        // Find target on same tile
        for (mut structure, build_pos) in &mut buildings {
            if flora_pos == build_pos {
                structure.current_hp = (structure.current_hp - flora.damage).max(0.0);
            }
        }
    }
}
```

### 4. Designation Type

Add `DesignationType::ClearFlora` to `src/layer1/designation.rs`.

## REFACTOR Phase: Quality & Design

- **Spatial Hash**: Use `OccupiedTiles` resource instead of building HashSet every frame.
- **Visuals**: Add rendering support (`Map::get_flora_char`).
- **Designation**: Integrate with `ClearFlora` designation to allow Pops to remove it.
- **Resources**: Clearing Flora should maybe drop `Biomass` or `Slime`.

## Acceptance Criteria

- [ ] `Flora` component exists.
- [ ] Flora spreads to adjacent empty tiles over time.
- [ ] Flora damages buildings on the same tile.
- [ ] Tests pass.

## Technical Guidance

- Use `OccupiedTiles` if available (from Spec 006) to check for buildings, but Flora might overlay buildings (attack them), so they can coexist.
- Flora should *not* coexist with other Flora.
- Ensure `flora_attack_system` is efficient (don't iterate all buildings inside all flora loop). Use a spatial lookup or sort.

## Questions

*Builder: add questions here if spec is unclear.*
