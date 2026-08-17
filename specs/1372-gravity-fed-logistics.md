# 1372: Gravity-Fed Logistics

## 1. Overview
**Layer:** 1

**Fantasy:** Water flows downhill.

**Mechanic:** Liquids and items move automatically "Down" Z-levels without power. Pumping them "Up" requires energy. Designing your base vertically allows for zero-energy transport chains.

**Emergence:** You build your reservoir at the top of the mountain. A sabotage blows the dam. The entire base is washed away because you put everything downhill for efficiency.

**Tension:** Free transport (Gravity) vs. Risk of cascading failure (Flooding).

## 2. Dependencies
- Base ECS system
- Conveyor/Pipe logistics system
- 3D Grid / Z-level system
- Energy/Power system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, process_gravity_logistics_system);
        app
    }

    #[test]
    fn test_items_move_down_without_power() {
        let mut app = setup_app();

        // Node A is above Node B
        let node_b = app.world_mut().spawn((
            LogisticsNode { z_level: 0, inventory: 0 },
        )).id();

        let node_a = app.world_mut().spawn((
            LogisticsNode { z_level: 1, inventory: 5 },
            Connection { target: node_b },
            Powered { is_powered: false }, // No power!
        )).id();

        app.update();

        // Item should move down to B
        let a_data = app.world().get::<LogisticsNode>(node_a).unwrap();
        let b_data = app.world().get::<LogisticsNode>(node_b).unwrap();

        assert_eq!(a_data.inventory, 4);
        assert_eq!(b_data.inventory, 1);
    }

    #[test]
    fn test_items_cannot_move_up_without_power() {
        let mut app = setup_app();

        // Node A is below Node B
        let node_b = app.world_mut().spawn((
            LogisticsNode { z_level: 1, inventory: 0 },
        )).id();

        let node_a = app.world_mut().spawn((
            LogisticsNode { z_level: 0, inventory: 5 },
            Connection { target: node_b },
            Powered { is_powered: false }, // No power!
        )).id();

        app.update();

        // Item should NOT move up to B
        let a_data = app.world().get::<LogisticsNode>(node_a).unwrap();
        let b_data = app.world().get::<LogisticsNode>(node_b).unwrap();

        assert_eq!(a_data.inventory, 5);
        assert_eq!(b_data.inventory, 0);
    }

    #[test]
    fn test_items_move_up_with_power() {
        let mut app = setup_app();

        // Node A is below Node B
        let node_b = app.world_mut().spawn((
            LogisticsNode { z_level: 1, inventory: 0 },
        )).id();

        let node_a = app.world_mut().spawn((
            LogisticsNode { z_level: 0, inventory: 5 },
            Connection { target: node_b },
            Powered { is_powered: true }, // Has power!
        )).id();

        app.update();

        // Item should move up to B
        let a_data = app.world().get::<LogisticsNode>(node_a).unwrap();
        let b_data = app.world().get::<LogisticsNode>(node_b).unwrap();

        assert_eq!(a_data.inventory, 4);
        assert_eq!(b_data.inventory, 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct LogisticsNode {
    pub z_level: i32,
    pub inventory: i32,
}

#[derive(Component)]
pub struct Connection {
    pub target: Entity,
}

#[derive(Component)]
pub struct Powered {
    pub is_powered: bool,
}

pub fn process_gravity_logistics_system(
    query: Query<(Entity, &LogisticsNode, &Connection, Option<&Powered>)>,
    mut target_query: Query<&mut LogisticsNode>,
) {
    let mut transfers = Vec::new();

    for (source_entity, source_node, connection, powered) in query.iter() {
        if source_node.inventory <= 0 { continue; }

        if let Ok(target_node) = target_query.get(connection.target) {
            let is_downhill = source_node.z_level > target_node.z_level;
            let is_level = source_node.z_level == target_node.z_level;
            let has_power = powered.map_or(false, |p| p.is_powered);

            // Downhill is always free. Level/Uphill requires power.
            if is_downhill || has_power {
                transfers.push((source_entity, connection.target));
            }
        }
    }

    // Execute transfers safely using get_many_mut to appease the borrow checker
    for (source_entity, target_entity) in transfers {
        if let Ok([mut source_node, mut target_node]) = target_query.get_many_mut([source_entity, target_entity]) {
            source_node.inventory -= 1;
            target_node.inventory += 1;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Safe Transfers:** Refactor the transfer logic to use a two-pass system or custom commands to safely mutate both the source and target nodes' inventories.
- **Fluid Dynamics:** If implementing for liquids, consider flow rate and pressure limits based on the height difference.
- **Level Transport:** Define whether horizontal (level) transport is free, requires minimal power, or uses momentum.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Items move from a higher `z_level` node to a lower `z_level` node regardless of `Powered` state.
- [ ] Items do not move from a lower/equal `z_level` to a higher/equal `z_level` unless `Powered.is_powered` is true.

## 7. Technical Guidance
- The main challenge is safely mutating two entities within the same system. Using a `SystemState` or passing transfer requests via Bevy `Event`s is the standard approach.

## 8. Questions
*Builder: add questions here if spec is unclear.*
