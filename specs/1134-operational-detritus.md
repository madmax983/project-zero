# 1134: Operational Detritus

## Overview

A lived-in world is messy. The friction of existence. High-activity tiles accumulate "Clutter" (trash, dust, loose wires) over time. Clutter slows movement and lowers beauty but increases "Scavenge" chance for free scrap/components. Requires "Janitor" jobs to clear.

## Dependencies

- None strictly, though depends on existing `TerrainGrid` and pathfinding systems in Layer 1.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_clutter_accumulation_on_high_activity() {
        // Arrange: Setup world with a TerrainGrid and an entity moving back and forth
        let mut app = App::new();
        // Setup ...

        // Act: Simulate several ticks of movement on specific tiles

        // Assert: Verify that the Clutter component on those tiles increases
    }

    #[test]
    fn test_clutter_slows_movement() {
        // Arrange: Setup a tile with high Clutter

        // Act: Calculate pathfinding cost or move speed through the tile

        // Assert: Verify movement speed is reduced compared to a clean tile
    }

    #[test]
    fn test_janitor_clears_clutter() {
        // Arrange: Setup tile with Clutter and a Pop with Janitor job

        // Act: Run the work_execution_system

        // Assert: Verify the Clutter is reduced to zero or near zero
    }

    #[test]
    fn test_scavenge_chance_on_clutter() {
        // Arrange: Tile with high clutter, pop searching/scavenging

        // Act: Scavenge action

        // Assert: High clutter provides scrap/component reward
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to pass the tests

#[derive(Component, Default)]
pub struct Clutter {
    pub level: f32,
}

// System to add clutter when activity happens
pub fn clutter_accumulation_system(
    mut query: Query<&mut Clutter>,
    activity_events: EventReader<ActivityEvent>,
) {
    // Increment clutter based on activity
}

// Update movement cost based on clutter
pub fn get_tile_movement_cost(clutter: &Clutter) -> f32 {
    1.0 + (clutter.level * 0.1) // Basic slowing effect
}
```

## 5. REFACTOR Phase: Quality & Design

- Ensure `Clutter` component is not added to every tile if not necessary (sparse storage).
- Balance the accumulation rate to not overwhelm the janitors.
- Hook into the existing Beauty calculation to lower room/area beauty based on Clutter level.
- Ensure the scavenger logic integrates with the existing item drop/loot system.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Clutter naturally accumulates in high-traffic areas over time.

## 7. Technical Guidance

- Use an event-based approach or piggyback on the movement system to track "activity" on tiles.
- `Clutter` should probably cap out at a maximum level.
- Consider adding a visual indicator (sprite change or overlay) when Clutter reaches certain thresholds.

## 8. Questions

*Builder: add questions here if spec is unclear.*
