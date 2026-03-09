# 228: Pneumatic Tubes

## 1. Overview

Introduces a rapid, point-to-point logistic system for transporting small, lightweight items (`BuildingPermit`, `GeneSample`, `ResearchData`, `KeyCard`) between buildings without using Pops.

Pneumatic Tubes consist of **Terminals** (Send/Receive points) and **Tube Segments** (Piping). Items travel instantly or very quickly through the network, consuming Power. Heavy usage can cause "Clogs" which require maintenance.

This system solves the "Paperwork Physicality" bottleneck (Spec 222) where critical permits are delayed by slow haulers.

### New Buildings
- **Tube Terminal**: A small building add-on or standalone structure that acts as an entry/exit point.
- **Tube Segment**: A constructible utility line (like Power/Pipes) that connects Terminals.

## 2. Dependencies

- `specs/006-building-placement.md` (Placement logic)
- `specs/042-energy-system.md` (Power consumption)
- `specs/143-data-physicality.md` (Data items)
- `specs/222-paperwork-physicality.md` (Permit items)

## 3. RED Phase: Tests First

```rust
// src/layer1/logistics/pneumatic_tubes_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType, Direction};
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::inventory::Inventory;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::logistics::pneumatic::{
        PneumaticTerminal, PneumaticTube, TubeNetwork, TubeCarrier,
        tube_transport_system, tube_clog_system
    };

    #[test]
    fn test_tube_network_connection() {
        let mut world = World::new();
        // Setup Terminal A at (0,0)
        let term_a = world.spawn((
            PneumaticTerminal { id: 1, connected_to: vec![] },
            GridPosition { x: 0, y: 0 },
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        // Setup Terminal B at (2,0)
        let term_b = world.spawn((
            PneumaticTerminal { id: 2, connected_to: vec![] },
            GridPosition { x: 2, y: 0 },
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        // Setup Tube Segments at (0,0), (1,0), (2,0)
        world.spawn((PneumaticTube, GridPosition { x: 0, y: 0 }));
        world.spawn((PneumaticTube, GridPosition { x: 1, y: 0 }));
        world.spawn((PneumaticTube, GridPosition { x: 2, y: 0 }));

        // Run network discovery system
        // Note: In real ECS, this might be reactive. For test, we assume a system rebuilds graph.
        // update_tube_network(&mut world);

        // Assert connectivity
        let a = world.get::<PneumaticTerminal>(term_a).unwrap();
        assert!(a.connected_to.contains(&2));
    }

    #[test]
    fn test_tube_sends_item() {
        let mut world = World::new();
        // Terminals A and B connected
        let term_a = world.spawn((
            PneumaticTerminal { id: 1, connected_to: vec![2] },
            GridPosition { x: 0, y: 0 },
            Inventory::default(), // Source inventory
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        let term_b = world.spawn((
            PneumaticTerminal { id: 2, connected_to: vec![1] },
            GridPosition { x: 10, y: 0 },
            Inventory::default(), // Dest inventory
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        // Add Permit to Terminal A
        let permit = world.spawn(Item { item_type: ItemType::BuildingPermit }).id();
        world.get_mut::<Inventory>(term_a).unwrap().add(permit);

        // Initiate Send Order
        world.entity_mut(term_a).insert(TubeCarrier {
            target_terminal_id: 2,
            item_entity: permit,
            progress: 0.0,
            speed: 2.0, // 2 tiles per tick
        });

        // Run transport system
        tube_transport_system(&mut world);

        // Check if item left A
        assert!(world.get::<Inventory>(term_a).unwrap().is_empty());

        // Run more ticks to arrive (Distance 10 / Speed 2 = 5 ticks)
        for _ in 0..5 {
            tube_transport_system(&mut world);
        }

        // Check if item arrived at B
        assert!(!world.get::<Inventory>(term_b).unwrap().is_empty());
        let item = world.get::<Inventory>(term_b).unwrap().items[0];
        assert_eq!(world.get::<Item>(item).unwrap().item_type, ItemType::BuildingPermit);
    }

    #[test]
    fn test_tube_clogging() {
        let mut world = World::new();
        // Terminal A
        let term_a = world.spawn((
            PneumaticTerminal { id: 1, connected_to: vec![2] },
            GridPosition { x: 0, y: 0 },
            TubeCarrier {
                target_terminal_id: 2,
                item_entity: Entity::from_raw(0),
                progress: 0.0,
                speed: 2.0
            },
            Inventory::default(),
            PowerConsumer { active: true, ..Default::default() },
        )).id();

        // Insert "Clog" component to a tube segment in the path
        let tube_segment = world.spawn((
            PneumaticTube,
            GridPosition { x: 1, y: 0 },
            super::Clogged { severity: 1.0 }
        )).id();

        // Run system
        tube_transport_system(&mut world);

        // Carrier should be stuck or rejected
        // For MVP: Carrier progress should not increase if next tile is clogged.
        let carrier = world.get::<TubeCarrier>(term_a).unwrap();
        assert_eq!(carrier.progress, 0.0, "Carrier should not move through clog");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### Components

```rust
// src/layer1/logistics/pneumatic.rs

#[derive(Component, Debug, Clone)]
pub struct PneumaticTerminal {
    pub id: u32,
    pub connected_to: Vec<u32>, // IDs of reachable terminals
}

#[derive(Component, Debug, Clone)]
pub struct PneumaticTube;

#[derive(Component, Debug, Clone)]
pub struct TubeCarrier {
    pub target_terminal_id: u32,
    pub item_entity: Entity,
    pub progress: f32, // Distance traveled
    pub speed: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Clogged {
    pub severity: f32, // 0.0 to 1.0
}
```

### Systems

```rust
// src/layer1/logistics/pneumatic.rs

pub fn tube_transport_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TubeCarrier, &GridPosition, &PneumaticTerminal)>,
    tubes: Query<(&GridPosition, Option<&Clogged>), With<PneumaticTube>>,
    mut terminals: Query<(Entity, &PneumaticTerminal, &mut Inventory)>,
) {
    for (carrier_entity, mut carrier, pos, start_term) in &mut query {
        // 1. Calculate path to target (Simplified: Manhattan or cached path)
        // 2. Check next tile for Clog
        // 3. Increment progress
        // 4. If progress >= distance, transfer item to target Inventory & despawn Carrier
    }
}

pub fn tube_clog_system(
    mut tubes: Query<&mut Clogged>,
    // random chance to clear or worsen
) {
    // Maintenance logic
}
```

## 5. REFACTOR Phase: Quality & Design

- **Pathfinding**: A* on the tube network is needed. Since tubes are static, the network graph can be cached in a `Resource`. Rebuild only when a `PneumaticTube` or `Terminal` is built/destroyed.
- **Visuals**: Carrier entity should be visible moving along the tube (or invisible if inside).
- **Item Restrictions**: Only allow `ItemType::BuildingPermit`, `ItemType::GeneSample`, etc. Reject `ItemType::SteelBeam` (too heavy).
- **Power**: Terminals consume high power only when sending/receiving (burst consumption).

## 6. Acceptance Criteria

- [ ] `PneumaticTerminal` and `PneumaticTube` components defined.
- [ ] Network discovery test passes (terminals find each other).
- [ ] Item transport test passes (A -> B).
- [ ] Clog logic prevents movement.
- [ ] Power consumption logic is integrated.
- [ ] Only "Small" items can be transported.

## 7. Technical Guidance

- Use a `Graph<u32, f32>` (petgraph or simple adj list) stored in a `TubeNetwork` resource for connectivity.
- `TubeCarrier` is a transient component attached to the *Terminal* initially, then maybe spawns a visual entity, but logically the item is "in the tube".
- Be careful with `Entity` references for items. If the item is "in the tube", it shouldn't be in any Inventory. It should be "held" by the Carrier.

## 8. Questions

- *Builder*: Can tubes go through walls?
- *Architect:* Yes, they are utilities like pipes.
- *Builder*: Can players intercept tubes?
- *Architect:* No, items are secure inside.
- *Builder*: Do clogs clear automatically?
- *Architect:* No, requires a "Maintenance" job (future) or "Purge" action (power cost). For now, just make them persist until fixed (or minimal decay).
