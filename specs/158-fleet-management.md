# 158: Fleet Management (Merge & Split)

## 1. Overview

As fleets grow and strategic needs change, players must be able to reorganize their naval forces.
This feature enables:
1.  **Merging**: Combining two fleets at the same location into a single fleet.
2.  **Splitting**: Detaching specific ships and cargo from a fleet to form a new fleet.

This provides the granular control needed for advanced Layer 2 gameplay (e.g., detaching a fast scout from a slow transport convoy, or consolidating mining fleets).

## 2. Dependencies

- `specs/099-fleet-movement.md` (Fleet Entity, InOrbit)
- `specs/157-ship-classes.md` (FleetComposition, Ship)
- `specs/101-system-mining.md` (FleetCargo)

## 3. RED Phase: Tests First

Write these tests in `src/layer2/fleet_management_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetComposition, InOrbit, FleetOrder};
    use crate::layer2::ship::{Ship, ShipType};
    use crate::layer2::mining::{FleetCargo, CargoStack};
    use crate::layer1::resources::ResourceType;
    use crate::layer2::management::{FleetSplitOrder, FleetMergeOrder, fleet_management_system};

    fn setup_world() -> World {
        let mut world = World::new();
        // Register components...
        world
    }

    #[test]
    fn test_merge_fleets() {
        let mut world = setup_world();
        let planet = world.spawn_empty().id();

        // Fleet A (Target)
        let fleet_a = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetComposition {
                ships: vec![Ship::new(ShipType::Scout)],
            },
            FleetCargo::default(),
        )).id();

        // Fleet B (Source - to be merged into A)
        let fleet_b = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetComposition {
                ships: vec![Ship::new(ShipType::Miner)],
            },
            FleetCargo {
                contents: vec![CargoStack { resource_type: ResourceType::Ore, amount: 10.0 }],
                capacity: 100.0,
            },
        )).id();

        // Issue Merge Order to Fleet B -> Merge into Fleet A
        world.entity_mut(fleet_b).insert(FleetOrder::Merge(FleetMergeOrder { target: fleet_a }));

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_management_system);
        schedule.run(&mut world);

        // Verify Fleet A has both ships
        let comp_a = world.get::<FleetComposition>(fleet_a).unwrap();
        assert_eq!(comp_a.ships.len(), 2);
        assert!(comp_a.ships.iter().any(|s| s.ship_type == ShipType::Scout));
        assert!(comp_a.ships.iter().any(|s| s.ship_type == ShipType::Miner));

        // Verify Fleet A has cargo
        let cargo_a = world.get::<FleetCargo>(fleet_a).unwrap();
        assert_eq!(cargo_a.current_load(), 10.0);

        // Verify Fleet B is despawned
        assert!(world.get_entity(fleet_b).is_none());
    }

    #[test]
    fn test_merge_fleets_fail_different_locations() {
        let mut world = setup_world();
        let planet_1 = world.spawn_empty().id();
        let planet_2 = world.spawn_empty().id();

        let fleet_a = world.spawn((
            Fleet,
            InOrbit { parent: planet_1 },
            FleetComposition::default(),
            FleetCargo::default(),
        )).id();

        let fleet_b = world.spawn((
            Fleet,
            InOrbit { parent: planet_2 }, // Different location
            FleetComposition::default(),
            FleetCargo::default(),
        )).id();

        world.entity_mut(fleet_b).insert(FleetOrder::Merge(FleetMergeOrder { target: fleet_a }));

        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_management_system);
        schedule.run(&mut world);

        // Verify Merge did NOT happen
        assert!(world.get_entity(fleet_b).is_some());
    }

    #[test]
    fn test_split_fleet() {
        let mut world = setup_world();
        let planet = world.spawn_empty().id();

        // Fleet A with 2 Scouts and 1 Miner
        let fleet_a = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetComposition {
                ships: vec![
                    Ship::new(ShipType::Scout),
                    Ship::new(ShipType::Scout),
                    Ship::new(ShipType::Miner),
                ],
            },
            FleetCargo::default(),
        )).id();

        // Split Logic: Move 1 Scout and 0 Cargo to new fleet
        // Index based splitting for MVP? Or Type count?
        // Let's use indices for precision.
        let split_order = FleetSplitOrder {
            ship_indices: vec![0], // Move the first ship
            cargo_transfer: vec![], // No cargo
        };

        world.entity_mut(fleet_a).insert(FleetOrder::Split(split_order));

        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_management_system);
        schedule.run(&mut world);

        // Verify Fleet A has 2 ships remaining
        let comp_a = world.get::<FleetComposition>(fleet_a).unwrap();
        assert_eq!(comp_a.ships.len(), 2);

        // Verify New Fleet spawned
        let mut fleets = world.query_filtered::<&FleetComposition, (With<Fleet>, Without<FleetOrder>)>();
        // Exclude fleet_a from query if needed, or check count
        // fleet_a should have consumed order, so Without<FleetOrder> matches both?
        // Actually fleet_a just lost the order.
        // We expect 2 fleets total.
        assert_eq!(fleets.iter(&world).count(), 2);

        // Find the new fleet
        let new_fleet = fleets.iter(&world)
            .find(|comp| comp.ships.len() == 1)
            .expect("New fleet should have 1 ship");

        assert_eq!(new_fleet.ships[0].ship_type, ShipType::Scout);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Update `FleetOrder` (`src/layer2/fleet.rs`)

```rust
use crate::layer2::management::{FleetMergeOrder, FleetSplitOrder};

#[derive(Component, Debug, Clone)]
pub enum FleetOrder {
    MoveTo(Entity),
    Mine(Entity),
    BuildStation(StationType),
    Merge(FleetMergeOrder), // New
    Split(FleetSplitOrder), // New
}
```

### 2. Define Management Structs & System (`src/layer2/management.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer2::fleet::{Fleet, FleetOrder, FleetComposition, InOrbit};
use crate::layer2::mining::{FleetCargo, CargoStack};
use crate::layer2::ship::Ship;

#[derive(Debug, Clone)]
pub struct FleetMergeOrder {
    pub target: Entity,
}

#[derive(Debug, Clone)]
pub struct FleetSplitOrder {
    pub ship_indices: Vec<usize>, // Indices in source fleet's ship list
    pub cargo_transfer: Vec<CargoStack>,
}

pub fn fleet_management_system(
    mut commands: Commands,
    mut fleets: Query<(Entity, &mut FleetOrder, &InOrbit, &mut FleetComposition, &mut FleetCargo)>,
    // We need a separate query to look up targets without borrowing conflict?
    // Bevy allows `fleets.get_mut(target)` inside the loop if we are careful,
    // but `fleets.iter_mut()` locks the whole query.
    // Use `ParamSet` or collect commands to apply later?
    // For MVP, simplistic approach:
) {
    // We can't iterate mutably and get_mut other entities easily in one pass.
    // 1. Collect orders.
    // 2. Execute.

    // ... Implementation detail: Using `combinations_mut` or separating queries is hard in Bevy simple systems.
    // Better approach:
    // Query for fleets with orders.
    // Query for "other fleets" (read-only or unsafe cell? No).

    // Workaround:
    // Iterate all fleets, find those with Merge/Split orders.
    // Since Merge involves two entities, we need exclusive world access or careful query usage.
    // Let's assume we use `unsafe` or `ParamSet` logic in real impl, but here is the logic structure.

    // Pseudo-code for Merge:
    // 1. Check if Source and Target are InOrbit of same parent.
    // 2. Move ships from Source.ships to Target.ships.
    // 3. Move cargo from Source.cargo to Target.cargo.
    // 4. Despawn Source.

    // Pseudo-code for Split:
    // 1. Create NewFleet at same Orbit.
    // 2. Remove specified indices from Source.ships, add to NewFleet.
    // 3. Remove specified cargo from Source.cargo, add to NewFleet.
    // 4. Remove Order.
}
```

### 3. Implementation Note for Builder

To solve the "Mutable Aliasing" problem in Bevy when merging two entities of the same query:
- Use `commands` to perform the merge deferred? No, logic is complex.
- Use `SystemState`?
- **Recommended**:
    - Iterate `(Entity, &FleetOrder)` to find actors.
    - Collect `(Source, Target)` pairs.
    - End borrow.
    - Loop pairs: `get_mut(source)` and `get_mut(target)`. Bevy `Query::get_many_mut([e1, e2])` allows this!

```rust
// In src/layer2/management.rs

pub fn fleet_management_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FleetOrder, &InOrbit, &mut FleetComposition, &mut FleetCargo), With<Fleet>>,
) {
    // Handle Split (Single entity)
    // ...

    // Handle Merge (Two entities)
    // 1. Identify merge candidates
    let mut merges = Vec::new();
    for (entity, order, orbit, _, _) in query.iter() {
        if let FleetOrder::Merge(merge_order) = order {
            merges.push((entity, merge_order.target));
        }
    }

    // 2. Execute merges
    for (source_e, target_e) in merges {
        if let Ok([mut source, mut target]) = query.get_many_mut([source_e, target_e]) {
            // Check locations match
            if source.2.parent == target.2.parent {
                // Move Ships
                let mut ships_to_move = std::mem::take(&mut source.3.ships);
                target.3.ships.append(&mut ships_to_move);

                // Move Cargo
                let cargo_to_move = std::mem::take(&mut source.4.contents);
                for stack in cargo_to_move {
                    target.4.add(stack.resource_type, stack.amount);
                }

                // Despawn Source
                commands.entity(source_e).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Validation**: Ensure `ship_indices` in Split are valid and sorted descending to avoid index shifting issues during removal.
- **Cargo Capacity**: Check if Target fleet has capacity for merged cargo. If not, what? Spill? Prevent merge?
    - *Decision*: Allow merge, but clamp cargo or expand capacity (if capacity is sum of ships).
    - Since `FleetComposition` determines capacity, merging ships *increases* capacity instantly. So it should fit.
- **Empty Fleets**: Ensure Split doesn't leave source fleet empty (or if it does, despawn it).

## 6. Acceptance Criteria

- [ ] `FleetMergeOrder` and `FleetSplitOrder` structs defined.
- [ ] `FleetOrder` updated.
- [ ] System handles `get_many_mut` for merging correctly.
- [ ] Ships and Cargo are preserved during merge/split.
- [ ] Source fleet is despawned after successful merge.
- [ ] Tests pass.

## 7. Technical Guidance

- Use `query.get_many_mut([e1, e2])` to safely access two mutable entities from the same query. It returns `Result<[Mut<T>; 2], ...>`.
- For splitting, when removing multiple indices from a Vec, sort indices descending so removing `idx[5]` doesn't invalidate `idx[2]`.
- Remember to `remove::<FleetOrder>` after processing split.
