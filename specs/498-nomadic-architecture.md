# Specification 498: Nomadic Architecture

## 1. Overview
Certain buildings, or entire foundation platforms, can be fitted with massive "Crawler Treads." These structures can physically move across the Layer 1 map, albeit very slowly and consuming massive fuel. They crush anything in their path.

## 2. Dependencies
- Building System
- Layer 1 Pathfinding / Grid Update
- Resource System (Fuel)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_mobile_building_moves_and_consumes_fuel() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Spawn a mobile building at (10, 10) with fuel
    let building = app.world_mut().spawn((
        Building::new(BuildingType::CrawlerBase),
        GridPosition { x: 10, y: 10 },
        MobileStructure { move_cost: 10.0 },
        Inventory { items: vec![Item::new(ItemType::Fuel, 50)] },
    )).id();

    // Act: Order it to move to (11, 10)
    app.world_mut().send_event(MoveStructureEvent {
        structure: building,
        target: GridPosition { x: 11, y: 10 },
    });
    app.update();

    // Assert: Position is updated and fuel is consumed
    let pos = app.world().get::<GridPosition>(building).unwrap();
    let inv = app.world().get::<Inventory>(building).unwrap();

    assert_eq!(pos.x, 11);
    assert_eq!(inv.get_count(ItemType::Fuel), 40);
}

#[test]
fn test_mobile_building_crushes_obstacles_in_path() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Spawn a mobile building at (10, 10)
    let building = app.world_mut().spawn((
        Building::new(BuildingType::CrawlerBase),
        GridPosition { x: 10, y: 10 },
        MobileStructure { move_cost: 0.0 },
    )).id();

    // Spawn an obstacle (tree) at (11, 10)
    let tree = app.world_mut().spawn((
        Flora { type_: FloraType::Tree },
        GridPosition { x: 11, y: 10 },
    )).id();

    // Act: Order the building to move into the tree
    app.world_mut().send_event(MoveStructureEvent {
        structure: building,
        target: GridPosition { x: 11, y: 10 },
    });
    app.update();

    // Assert: The tree is destroyed and the building occupies the tile
    assert!(app.world().get_entity(tree).is_none());
    let pos = app.world().get::<GridPosition>(building).unwrap();
    assert_eq!(pos.x, 11);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// Process `MoveStructureEvent`. Deduct fuel. Delete any `Flora` or small `Debris` at target tile. Update `GridPosition` of structure.
```

## 5. REFACTOR Phase: Quality & Design
- Update the Grid occupancy map safely when a multi-tile building moves.
- Ensure movement speed scales correctly with the building's size and fuel consumption.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Buildings with crawler treads can be moved and correctly update their footprint on the grid.

## 7. Technical Guidance
- Add a `MobileStructure` component to buildings capable of movement.
- Provide a `MoveStructureOrder` that functions similarly to a Pop's movement order but executes over multiple ticks (very slow) and requires `Fuel`.
- The building's grid occupancy must be cleared from the old footprint and applied to the new footprint precisely upon entering the new tile.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
