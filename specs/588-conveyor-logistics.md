# 588: Conveyor Logistics

## 1. Overview
**Layer:** 1
**Fantasy:** The transition from a village to a factory. Watching items flow like water.
**Mechanic:** Constructible "Conveyor Belts" and "Inserters" that move items between stockpiles and machines automatically, consuming power. They block pathfinding for Pops (unless "Underground" or "Overhead").

## 2. Dependencies
- Layer 1 Power Grid System
- Layer 1 Pathfinding/Navigation Mesh
- Layer 1 Inventory/Item System

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_conveyor_moves_item() {
        let mut app = App::new();
        let start_pos = Vec2::new(0.0, 0.0);
        let end_pos = Vec2::new(1.0, 0.0);

        let belt1 = app.world_mut().spawn((ConveyorBelt { direction: end_pos }, Position(start_pos))).id();
        let belt2 = app.world_mut().spawn((ConveyorBelt { direction: end_pos }, Position(end_pos))).id();

        let item = app.world_mut().spawn((Item, Position(start_pos), OnBelt(belt1))).id();

        app.add_systems(Update, conveyor_movement_system);
        app.update();

        let item_pos = app.world().get::<Position>(item).unwrap();
        assert_eq!(item_pos.0, end_pos, "Item should move to next belt tile");
        let item_belt = app.world().get::<OnBelt>(item).unwrap();
        assert_eq!(item_belt.0, belt2, "Item should be on the next belt");
    }

    #[test]
    fn test_unpowered_conveyor_halts() {
        let mut app = App::new();
        let start_pos = Vec2::new(0.0, 0.0);
        let end_pos = Vec2::new(1.0, 0.0);

        let belt1 = app.world_mut().spawn((
            ConveyorBelt { direction: end_pos },
            Position(start_pos),
            RequiresPower { powered: false }
        )).id();

        let item = app.world_mut().spawn((Item, Position(start_pos), OnBelt(belt1))).id();

        app.add_systems(Update, conveyor_movement_system);
        app.update();

        let item_pos = app.world().get::<Position>(item).unwrap();
        assert_eq!(item_pos.0, start_pos, "Item should not move on unpowered belt");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct ConveyorBelt { pub direction: Vec2 }

#[derive(Component)]
pub struct OnBelt(pub Entity);

#[derive(Component)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct RequiresPower { pub powered: bool }

pub fn conveyor_movement_system(
    mut items: Query<(Entity, &mut Position, &mut OnBelt)>,
    belts: Query<(Entity, &ConveyorBelt, &Position, Option<&RequiresPower>), Without<OnBelt>>,
) {
    for (item_ent, mut item_pos, mut on_belt) in items.iter_mut() {
        if let Ok((_, belt, _belt_pos, power)) = belts.get(on_belt.0) {
            if let Some(p) = power {
                if !p.powered { continue; }
            }

            let target_pos = belt.direction;
            item_pos.0 = target_pos;

            // Find next belt
            for (next_belt_ent, _, next_belt_pos, _) in belts.iter() {
                if next_belt_pos.0 == target_pos {
                    on_belt.0 = next_belt_ent;
                    break;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** The nested loop searching for `next_belt_ent` is `O(N^2)`. Use a spatial hashmap resource (`Res<Grid>`) to instantly look up the entity at `target_pos`.
- **Performance:** Moving every item individually is expensive. Items on belts should probably be managed as a queue or array within the `ConveyorBelt` entity itself rather than discrete components.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Items move along connected belts.
- [ ] Belts without power halt all item movement.
- [ ] Belts correctly modify the navigation mesh to block Pop pathfinding.

## 7. Technical Guidance
- Be extremely careful with cyclic belts (loops). Ensure the movement system can handle full loops without deadlocking.
- Integrate with the `update_navigation_mesh_system` to ensure standard ground belts are marked as non-walkable.

## 8. Questions
*Builder: add questions here if spec is unclear.*
