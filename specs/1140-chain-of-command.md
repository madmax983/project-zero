# Chain of Command

## 1. Overview
Orders take time to travel down the line. Combat units have a "Command Radius". Officers relay player orders to nearby troops. Troops out of radius or with dead officers revert to "Instinct" (Flee, Charge, Hunker) based on traits.

## 2. Dependencies
- Core unit components (Health, Position)
- Combat system framework

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_unit_receives_orders_in_command_radius() {
        let mut world = World::new();
        let officer_id = world.spawn((Position { x: 0.0, y: 0.0 }, Officer { command_radius: 10.0 })).id();
        let soldier_id = world.spawn((Position { x: 5.0, y: 0.0 }, Soldier { instinct: Instinct::Charge }, CurrentOrder::None)).id();

        world.insert_resource(PlayerCommand { order: Order::HoldLine, target_officer: officer_id });
        world.run_system_once(propagate_orders_system);

        let order = world.get::<CurrentOrder>(soldier_id).unwrap();
        assert_eq!(*order, CurrentOrder::Commanded(Order::HoldLine));
    }

    #[test]
    fn test_unit_reverts_to_instinct_out_of_radius() {
        let mut world = World::new();
        let officer_id = world.spawn((Position { x: 0.0, y: 0.0 }, Officer { command_radius: 10.0 })).id();
        let soldier_id = world.spawn((Position { x: 15.0, y: 0.0 }, Soldier { instinct: Instinct::Flee }, CurrentOrder::Commanded(Order::HoldLine))).id();

        world.insert_resource(PlayerCommand { order: Order::Attack, target_officer: officer_id });
        world.run_system_once(propagate_orders_system);

        let order = world.get::<CurrentOrder>(soldier_id).unwrap();
        assert_eq!(*order, CurrentOrder::InstinctDriven(Instinct::Flee));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct Position { pub x: f32, pub y: f32 }

#[derive(Component)]
pub struct Officer { pub command_radius: f32 }

#[derive(Component)]
pub struct Soldier { pub instinct: Instinct }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Instinct { Charge, Flee, Hunker }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Order { HoldLine, Attack }

#[derive(Component, PartialEq, Debug)]
pub enum CurrentOrder { None, Commanded(Order), InstinctDriven(Instinct) }

#[derive(Resource)]
pub struct PlayerCommand { pub order: Order, pub target_officer: Entity }

pub fn propagate_orders_system(
    command: Option<Res<PlayerCommand>>,
    officers: Query<(&Position, &Officer)>,
    mut soldiers: Query<(&Position, &Soldier, &mut CurrentOrder)>,
) {
    if let Some(cmd) = command {
        if let Ok((off_pos, officer)) = officers.get(cmd.target_officer) {
            for (sol_pos, soldier, mut order) in soldiers.iter_mut() {
                let dx = off_pos.x - sol_pos.x;
                let dy = off_pos.y - sol_pos.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= officer.command_radius {
                    *order = CurrentOrder::Commanded(cmd.order);
                } else {
                    *order = CurrentOrder::InstinctDriven(soldier.instinct);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Spatial hashing or octree for command radius checks instead of O(N) distance checks.
- **Improvements:** Make orders an Event instead of a Resource so multiple officers can receive orders simultaneously. Delay in order propagation.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Soldiers within an officer's radius receive the officer's commands.
- [ ] Soldiers outside the radius fall back to instinct.

## 7. Technical Guidance
- Start with a simple distance check, then optimize with Bevy's spatial query tools if we run into performance issues.
- Keep the `Instinct` enum flexible for future personality traits.

## 8. Questions
*Builder: add questions here if spec is unclear.*
