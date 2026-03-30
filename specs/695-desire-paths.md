# Spec 695: Desire Paths

## 1. Overview
The colony is shaped by its inhabitants, not just the architect. Walking on natural terrain (Grass) slowly converts it to "Dirt" or "Path". Paths have higher walk speed than Grass but lower Beauty. Paved roads prevent wear but cost resources.

## 2. Dependencies
- `TerrainGrid`
- Movement/Pathfinding systems
- Pop tracking

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_walking_on_grass_accumulates_wear() {
        let mut app = App::new();
        // Setup TerrainGrid with Grass tiles
        // Move a Pop across a specific Grass tile multiple times
        // Assert that the tile's 'wear' value increases
    }

    #[test]
    fn test_high_wear_converts_grass_to_dirt_path() {
        let mut app = App::new();
        // Setup a Grass tile near the wear threshold
        // Apply final wear increment
        // Run systems
        // Assert the terrain type changes to Dirt/Path
    }

    #[test]
    fn test_paved_roads_do_not_accumulate_wear() {
        let mut app = App::new();
        // Setup a Paved Road tile
        // Move a Pop across it multiple times
        // Assert that no wear is accumulated and the tile type remains Paved Road
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add a wear tracking mechanism, potentially a new Grid or adding a field to TerrainGrid
#[derive(Component)]
pub struct TerrainWear(pub f32);

// System to increment wear based on entity movement events or position tracking
// System to transition terrain types when wear > threshold
```

## 5. REFACTOR Phase: Quality & Design
- Instead of adding a component to every tile, consider a sparse `HashMap` for tracking wear, or integrating directly into the `TerrainGrid` data structure if it's already an array of structs.
- Ensure pathfinding costs dynamically update when a path forms, encouraging more pops to use the new desire path.
- Add an extremely slow recovery mechanism so unused paths eventually grow back into grass.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Heavy traffic visibly and mechanically transforms the terrain over time.

## 7. Technical Guidance
- You will need to hook into the `movement_system` to register when a tile is traversed.
- Triggering a pathfinding graph rebuild might be necessary when terrain costs change, but be mindful of performance. Consider batching updates.

## 8. Questions
*Builder: add questions here if spec is unclear.*
