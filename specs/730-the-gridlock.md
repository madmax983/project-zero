# Spec 730: The Gridlock

## 1. Overview
**Layer:** 1
**Fantasy:** The claustrophobia of a busy station.
**Mechanic:** Tiles have a "Crowding" penalty. If multiple Pops occupy/traverse a tile in a short window, movement speed drops drastically. "Wide" corridors (3-tile) reduce this.
**Emergence:** Shift change at the mine causes a stampede in the single-tile hallway. Productivity drops by 15% just because people are bumping into each other. You are forced to bulldoze housing to widen the roads.
**Tension:** Space efficiency (narrow halls) vs. Flow efficiency (wide halls).

## 2. Dependencies
- Base simulation framework
- Grid and Pathfinding systems

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::movement::MovementSystem;

    #[test]
    fn test_crowding_penalty_applied() {
        // Arrange
        let mut grid = MovementGrid::new();
        let pos = GridPosition::new(5, 5);

        // Act
        // Simulate 5 pops moving through the same tile recently
        for _ in 0..5 {
            grid.record_movement(pos);
        }

        let cost = grid.get_movement_cost(pos);

        // Assert
        // Base cost is 1.0, with 5 pops it should be much higher
        assert!(cost > 2.0);
    }

    #[test]
    fn test_crowding_decays_over_time() {
        // Arrange
        let mut grid = MovementGrid::new();
        let pos = GridPosition::new(5, 5);
        grid.record_movement(pos);
        grid.record_movement(pos);

        let initial_cost = grid.get_movement_cost(pos);

        // Act
        grid.tick_decay(); // Simulate time passing

        // Assert
        let final_cost = grid.get_movement_cost(pos);
        assert!(final_cost < initial_cost);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use std::collections::HashMap;
use crate::layer1::map::GridPosition;

pub struct MovementGrid {
    pub recent_traffic: HashMap<GridPosition, u32>,
}

impl MovementGrid {
    pub fn new() -> Self {
        Self { recent_traffic: HashMap::new() }
    }

    pub fn record_movement(&mut self, pos: GridPosition) {
        *self.recent_traffic.entry(pos).or_insert(0) += 1;
    }

    pub fn get_movement_cost(&self, pos: GridPosition) -> f32 {
        let traffic = self.recent_traffic.get(&pos).unwrap_or(&0);
        1.0 + (*traffic as f32 * 0.5) // Example penalty scaling
    }

    pub fn tick_decay(&mut self) {
        // Simple decay
        self.recent_traffic.retain(|_, count| {
            if *count > 0 {
                *count -= 1;
                *count > 0
            } else {
                false
            }
        });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement decay cleanly using a timer or `ResMut<Time>` in a Bevy system.
- Optimize the `HashMap` into a dense 2D array or grid if performance becomes an issue during pathfinding.
- Integrate into A* pathfinding so Pops *avoid* crowded hallways if a slightly longer, empty path exists.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code

## 7. Technical Guidance
- Add `MovementGrid` as a resource. Update it in the `movement_system` when entities change tiles.
- The pathfinding algorithm needs to read this resource.

## 8. Questions
*Builder: add questions here if spec is unclear.*
