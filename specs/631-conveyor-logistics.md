# Spec 631: Conveyor Logistics

## 1. Overview
The transition from a village to a factory. Constructible "Conveyor Belts" and "Inserters" move items between stockpiles and machines automatically, consuming power. They block pathfinding for Pops (unless designated "Underground" or "Overhead"). This allows automation but introduces rigidity.

## 2. Dependencies
- `src/layer1/items.rs` (Items that can be moved)
- `src/layer1/grid.rs` (Grid system for placement and pathfinding)
- `src/layer1/power.rs` (Power consumption)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::grid::GridPosition;
    use crate::layer1::power::PowerConsumer;

    #[test]
    fn test_conveyor_moves_item_forward() {
        let mut app = App::new();
        app.add_systems(Update, conveyor_movement_system);

        let item_entity = app.world_mut().spawn((
            Item { item_type: ItemType::Ore },
            GridPosition { x: 0, y: 0 },
            OnConveyor,
        )).id();

        let conveyor_entity = app.world_mut().spawn((
            ConveyorBelt { direction: Direction::East },
            GridPosition { x: 0, y: 0 },
            PowerConsumer { consumed: 5.0, is_powered: true },
        )).id();

        app.update();

        let item_pos = app.world().get::<GridPosition>(item_entity).unwrap();
        assert_eq!(item_pos.x, 1, "Item should have moved East");
        assert_eq!(item_pos.y, 0);
    }

    #[test]
    fn test_unpowered_conveyor_does_not_move_item() {
        let mut app = App::new();
        app.add_systems(Update, conveyor_movement_system);

        let item_entity = app.world_mut().spawn((
            Item { item_type: ItemType::Ore },
            GridPosition { x: 0, y: 0 },
            OnConveyor,
        )).id();

        let conveyor_entity = app.world_mut().spawn((
            ConveyorBelt { direction: Direction::East },
            GridPosition { x: 0, y: 0 },
            PowerConsumer { consumed: 5.0, is_powered: false }, // Unpowered
        )).id();

        app.update();

        let item_pos = app.world().get::<GridPosition>(item_entity).unwrap();
        assert_eq!(item_pos.x, 0, "Item should not move if conveyor is unpowered");
        assert_eq!(item_pos.y, 0);
    }

    #[test]
    fn test_conveyor_blocks_pathfinding() {
        let conveyor = ConveyorBelt { direction: Direction::East };
        assert!(conveyor.blocks_pathfinding(), "Standard conveyors should block pathfinding");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::grid::GridPosition;
use crate::layer1::power::PowerConsumer;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Component)]
pub struct ConveyorBelt {
    pub direction: Direction,
}

impl ConveyorBelt {
    pub fn blocks_pathfinding(&self) -> bool {
        true
    }
}

#[derive(Component)]
pub struct OnConveyor;

pub fn conveyor_movement_system(
    conveyors: Query<(&ConveyorBelt, &GridPosition, &PowerConsumer)>,
    mut items: Query<(&mut GridPosition, &OnConveyor), Without<ConveyorBelt>>,
) {
    for (conveyor, conv_pos, power) in conveyors.iter() {
        if !power.is_powered {
            continue;
        }

        // Find items on this conveyor
        for (mut item_pos, _) in items.iter_mut() {
            if item_pos.x == conv_pos.x && item_pos.y == conv_pos.y {
                match conveyor.direction {
                    Direction::North => item_pos.y += 1,
                    Direction::South => item_pos.y -= 1,
                    Direction::East => item_pos.x += 1,
                    Direction::West => item_pos.x -= 1,
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Improve `conveyor_movement_system` to handle multiple items. Currently it moves them instantly, which might cause items to pile up or skip if not synchronized correctly with tick rate.
- Implement an `Inserter` entity to handle the pickup and drop-off of items onto/off of the belts.
- Update the Pathfinding system to respect the `blocks_pathfinding` flag on Grid tiles occupied by conveyors.
- Add "Underground" and "Overhead" belt variants that do not block pathfinding.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Items on powered conveyors move one tile in the conveyor's direction per update.
- [ ] Items on unpowered conveyors do not move.

## 7. Technical Guidance
- Movement should ideally be processed with a fixed timestep to ensure consistent flow rates (e.g., `FixedUpdate`).
- Items on conveyors should probably be stored in a collection within the Conveyor component or synchronized carefully if kept as separate entities to prevent overlapping.

## 8. Questions
*Builder: add questions here if spec is unclear.*
