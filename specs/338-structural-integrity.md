# 338 - Structural Integrity

## 1. Overview
The **Structural Integrity** feature implements a physics-based mining mechanic where removing too much supportive material (like rock pillars) causes the ceiling to collapse. This forces players to mine intelligently (e.g., pillar-and-stall techniques) rather than just designating massive empty rooms, adding a risk/reward element to resource extraction speed. Cave-ins will damage pops, destroy items, and block paths.

## 2. Dependencies
- `018` Mining and Resources
- `002` Basic Map (TerrainGrid)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_mining_reduces_support() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, mining_system);

        // A 3x3 solid rock area where the center is mined
        let grid = app.world_mut().spawn(TerrainGrid::new(3, 3)).id();
        let center_tile = app.world().get::<TerrainGrid>(grid).unwrap().get_tile(1, 1).unwrap();

        let initial_support = app.world().get::<SupportValue>(center_tile).unwrap().value;

        // Act - Mine the center tile
        app.world_mut().entity_mut(center_tile).insert(MinedStatus { is_mined: true });
        app.update();

        // Assert
        let final_support = app.world().get::<SupportValue>(center_tile).unwrap().value;
        assert!(final_support < initial_support, "Mining a tile should reduce its structural support.");
    }

    #[test]
    fn test_cave_in_triggers_below_threshold() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, cave_in_check_system);
        app.add_event::<CaveInEvent>();

        let tile_entity = app.world_mut().spawn((
            TileCoords { x: 5, y: 5 },
            SupportValue { value: 0.1, threshold: 0.2 }, // Support is below threshold
            MinedStatus { is_mined: true },
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<CaveInEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).any(|e| e.entity == tile_entity), "CaveInEvent should trigger for tiles below the support threshold.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct TileCoords {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct SupportValue {
    pub value: f32,
    pub threshold: f32,
}

#[derive(Component)]
pub struct MinedStatus {
    pub is_mined: bool,
}

#[derive(Event)]
pub struct CaveInEvent {
    pub entity: Entity,
    pub coords: TileCoords,
}

pub fn mining_system(
    mut tiles: Query<(&mut SupportValue, &MinedStatus), Changed<MinedStatus>>,
) {
    for (mut support, mined_status) in tiles.iter_mut() {
        if mined_status.is_mined {
            support.value = 0.0; // The mined tile provides 0 support
        }
    }
}

pub fn cave_in_check_system(
    tiles: Query<(Entity, &SupportValue, &TileCoords)>,
    mut cave_in_events: EventWriter<CaveInEvent>,
) {
    for (entity, support, coords) in tiles.iter() {
        if support.value < support.threshold {
            cave_in_events.send(CaveInEvent {
                entity,
                coords: TileCoords { x: coords.x, y: coords.y },
            });
            // Additional logic to handle the cave-in (e.g., changing terrain type, damage)
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
  - `SupportValue` should not just check the tile itself, but aggregate support from adjacent unmined tiles within a certain radius. A proper cellular automaton or diffusion algorithm would model support arcs realistically.
  - Separate the "checking" of support and the "execution" of the cave-in into different systems, linked by `CaveInEvent`.
- **Code Smells**:
  - The current implementation is too simplistic; mining a tile drops its own support to zero, but doesn't affect neighbors. The grid logic needs to calculate load bearing.
- **Performance**:
  - Recalculating the support grid on every tick is expensive. Only recalculate local support values when a `MinedStatus` changes (via `Changed` filter) or a support pillar is constructed/destroyed.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Mining a tile reduces local support, triggering cave-ins when support falls below a threshold.

## 7. Technical Guidance
- Implement a `calculate_local_support` helper function that checks a radius (e.g., 2-3 tiles) around a mined tile to see if nearby solid rock or constructed pillars are bearing the load.
- If a cave-in occurs, it should drop a "Rubble" item or change the terrain to a blocked state, requiring colonists to clear it.
- Pops caught in a cave-in radius should receive physical damage (hook into `034` Pop Health).

## 8. Questions
*Builder: add questions here if spec is unclear.*
