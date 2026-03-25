# Spec 607: Conveyor Logistics

## 1. Overview
**Layer:** 1
**Fantasy:** The transition from a village to a factory. Watching items flow like water.
**Mechanic:** Constructible "Conveyor Belts" and "Inserters" that move items between stockpiles and machines automatically, consuming power. They block pathfinding for Pops (unless "Underground" or "Overhead").
**Emergence:** You automate your entire food production. A power outage stops the belts. The food rots on the belt because no one can reach it to haul it manually.
**Tension:** Flexible manual labor (Pops) vs. Efficient but rigid automation (Belts).

## 2. Dependencies
- `src/layer1/map.rs` or `src/layer1/terrain.rs` (Pathfinding blocking mechanics)
- `src/layer1/inventory.rs` or `src/layer1/items.rs` (Item entity tracking and movement)
- `src/layer1/power.rs` or `src/layer1/energy.rs` (Power grid integration for consumption)
- `src/layer1/hauling.rs` (Alternative to manual pop hauling)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod conveyor_logistics_tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_conveyor_moves_item_forward() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_conveyor_belts_system);

        let item_entity = app.world_mut().spawn(Item).id();
        let belt_entity = app.world_mut().spawn((
            ConveyorBelt { direction: Direction::East, is_powered: true },
            GridPosition { x: 5, y: 5 },
            Inventory { item: Some(item_entity) }
        )).id();

        let target_belt_entity = app.world_mut().spawn((
            ConveyorBelt { direction: Direction::East, is_powered: true },
            GridPosition { x: 6, y: 5 },
            Inventory { item: None }
        )).id();

        // Need a resource or system parameter to track grid connectivity
        app.insert_resource(GridMap::new(vec![belt_entity, target_belt_entity]));

        // Act
        app.update();

        // Assert
        let belt_inv = app.world().get::<Inventory>(belt_entity).unwrap();
        let target_inv = app.world().get::<Inventory>(target_belt_entity).unwrap();

        assert!(belt_inv.item.is_none(), "Item should have moved off the first belt.");
        assert_eq!(target_inv.item, Some(item_entity), "Item should be on the target belt.");
    }

    #[test]
    fn test_unpowered_conveyor_does_not_move_item() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_conveyor_belts_system);

        let item_entity = app.world_mut().spawn(Item).id();
        let belt_entity = app.world_mut().spawn((
            ConveyorBelt { direction: Direction::East, is_powered: false }, // Unpowered
            GridPosition { x: 5, y: 5 },
            Inventory { item: Some(item_entity) }
        )).id();

        let target_belt_entity = app.world_mut().spawn((
            ConveyorBelt { direction: Direction::East, is_powered: true },
            GridPosition { x: 6, y: 5 },
            Inventory { item: None }
        )).id();

        app.insert_resource(GridMap::new(vec![belt_entity, target_belt_entity]));

        // Act
        app.update();

        // Assert
        let belt_inv = app.world().get::<Inventory>(belt_entity).unwrap();
        let target_inv = app.world().get::<Inventory>(target_belt_entity).unwrap();

        assert_eq!(belt_inv.item, Some(item_entity), "Item should remain on the unpowered belt.");
        assert!(target_inv.item.is_none(), "Target belt should remain empty.");
    }

    #[test]
    fn test_conveyor_blocks_pop_pathfinding() {
        // Arrange
        let mut app = App::new();
        let belt_entity = app.world_mut().spawn((
            ConveyorBelt { direction: Direction::North, is_powered: true },
            GridPosition { x: 10, y: 10 },
            PathingBlocker, // Ensures pathfinding engine ignores this tile
        )).id();

        // Act
        // Mock a pathfinding query or check the navigation grid update system.
        let is_passable = check_tile_passability(&app.world(), GridPosition { x: 10, y: 10 });

        // Assert
        assert!(!is_passable, "Conveyor belts should block pop pathfinding.");
    }

    #[test]
    fn test_inserter_moves_item_from_machine_to_belt() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_inserters_system);

        let item_entity = app.world_mut().spawn(Item).id();
        let machine_entity = app.world_mut().spawn((
            Machine,
            GridPosition { x: 2, y: 2 },
            Inventory { item: Some(item_entity) }
        )).id();

        let belt_entity = app.world_mut().spawn((
            ConveyorBelt { direction: Direction::East, is_powered: true },
            GridPosition { x: 4, y: 2 },
            Inventory { item: None }
        )).id();

        let inserter_entity = app.world_mut().spawn((
            Inserter { source_offset: IVec2::new(-1, 0), target_offset: IVec2::new(1, 0), is_powered: true },
            GridPosition { x: 3, y: 2 },
        )).id();

        app.insert_resource(GridMap::new(vec![machine_entity, belt_entity, inserter_entity]));

        // Act
        app.update();

        // Assert
        let machine_inv = app.world().get::<Inventory>(machine_entity).unwrap();
        let belt_inv = app.world().get::<Inventory>(belt_entity).unwrap();

        assert!(machine_inv.item.is_none(), "Inserter should pull item from machine.");
        assert_eq!(belt_inv.item, Some(item_entity), "Inserter should place item on the belt.");
    }
}

// Helper mock for testing
fn check_tile_passability(world: &World, pos: GridPosition) -> bool {
    for (_, p, blocker) in world.query::<(Entity, &GridPosition, Option<&PathingBlocker>)>().iter(world) {
        if p.x == pos.x && p.y == pos.y {
            return blocker.is_none();
        }
    }
    true
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub struct ConveyorBelt {
    pub direction: Direction,
    pub is_powered: bool,
}

#[derive(Component)]
pub struct Inserter {
    pub source_offset: IVec2,
    pub target_offset: IVec2,
    pub is_powered: bool,
}

#[derive(Component)]
pub struct Machine;

#[derive(Component)]
pub struct PathingBlocker;

#[derive(Component, Clone)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Inventory {
    pub item: Option<Entity>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn to_offset(&self) -> IVec2 {
        match self {
            Direction::North => IVec2::new(0, 1),
            Direction::South => IVec2::new(0, -1),
            Direction::East => IVec2::new(1, 0),
            Direction::West => IVec2::new(-1, 0),
        }
    }
}

#[derive(Resource)]
pub struct GridMap {
    pub entities: Vec<Entity>,
}

impl GridMap {
    pub fn new(entities: Vec<Entity>) -> Self {
        Self { entities }
    }

    pub fn get_entity_at(&self, world: &World, x: i32, y: i32) -> Option<Entity> {
        for &entity in &self.entities {
            if let Some(pos) = world.get::<GridPosition>(entity) {
                if pos.x == x && pos.y == y {
                    return Some(entity);
                }
            }
        }
        None
    }
}

pub fn process_conveyor_belts_system(world: &mut World) {
    let mut moves = Vec::new();

    // Find all ready moves
    let mut query = world.query::<(Entity, &ConveyorBelt, &GridPosition, &Inventory)>();
    for (entity, belt, pos, inv) in query.iter(world) {
        if !belt.is_powered || inv.item.is_none() {
            continue;
        }

        let offset = belt.direction.to_offset();
        let target_x = pos.x + offset.x;
        let target_y = pos.y + offset.y;

        let grid_map = world.resource::<GridMap>();
        if let Some(target_entity) = grid_map.get_entity_at(world, target_x, target_y) {
            moves.push((entity, target_entity, inv.item.unwrap()));
        }
    }

    // Apply moves (minimal naive implementation)
    for (source, target, item) in moves {
        if let Some(mut target_inv) = world.get_mut::<Inventory>(target) {
            if target_inv.item.is_none() {
                target_inv.item = Some(item);
                if let Some(mut source_inv) = world.get_mut::<Inventory>(source) {
                    source_inv.item = None;
                }
            }
        }
    }
}

pub fn process_inserters_system(world: &mut World) {
    let mut moves = Vec::new();

    let mut query = world.query::<(Entity, &Inserter, &GridPosition)>();
    for (_, inserter, pos) in query.iter(world) {
        if !inserter.is_powered { continue; }

        let source_x = pos.x + inserter.source_offset.x;
        let source_y = pos.y + inserter.source_offset.y;
        let target_x = pos.x + inserter.target_offset.x;
        let target_y = pos.y + inserter.target_offset.y;

        let grid_map = world.resource::<GridMap>();
        if let (Some(source_entity), Some(target_entity)) = (
            grid_map.get_entity_at(world, source_x, source_y),
            grid_map.get_entity_at(world, target_x, target_y)
        ) {
            moves.push((source_entity, target_entity));
        }
    }

    for (source, target) in moves {
        let mut item_to_move = None;
        if let Some(source_inv) = world.get::<Inventory>(source) {
            item_to_move = source_inv.item;
        }

        if let Some(item) = item_to_move {
            if let Some(mut target_inv) = world.get_mut::<Inventory>(target) {
                if target_inv.item.is_none() {
                    target_inv.item = Some(item);
                    if let Some(mut source_inv) = world.get_mut::<Inventory>(source) {
                        source_inv.item = None;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor Opportunity:** The `GridMap` and `world.get_mut` loop in the minimal implementation is highly inefficient and breaks Bevy's parallelism. Refactor to use Bevy's `Event` system for requesting item transfers, handled in a subsequent ordered system.
- **Refactor Opportunity:** Introduce a state enum for items on belts (`Moving`, `Blocked`, `PendingTransfer`) to allow smooth rendering of items between tiles and prevent instantaneous teleportation in the UI.
- **Pathfinding Updates:** Ensure `PathingBlocker` triggers a map-wide pathfinding nav-mesh rebuild when a conveyor belt is constructed or destroyed.
- **Optimization:** Use spatial hashing or a strict array-backed 2D grid component to lookup adjacent belts instead of iterating through entities.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Items flow consecutively across powered belts and stop at unpowered or blocked belts.
- [ ] Conveyor tiles cannot be path-found through by normal Pops.

## 7. Technical Guidance
- Implement this inside `src/layer1/logistics/conveyor.rs`.
- Handle edge cases: Multiple inserters trying to place an item on the exact same belt tile at the same time. Priority rules or random selection must apply.
- Bevy's ECS handles mutable borrows strictly; avoid querying for `&mut Inventory` twice in the same loop. Buffer the intended transfers and apply them safely.

## 8. Questions
*Builder: add questions here if spec is unclear.*
