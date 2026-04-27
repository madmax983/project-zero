# 1209: Asteroid Hollowing

## 1. Overview
**Layer:** 1

**Fantasy:** Living inside the rock. The ultimate bunker.

**Mechanic:** On Asteroid maps, you don't build *on* the surface; you mine *into* it. The asteroid shell provides massive Armor and Radiation shielding. However, mining too close to the edge weakens the "Hull Integrity".

**Emergence:** You get greedy and mine a thin wall for gold. A stray meteor punches through the weakened shell, venting the atmosphere.

**Tension:** Resources (mining the walls) vs. Safety (thick walls).

## 2. Dependencies
- Terrain system
- Asteroid maps

## 3. RED Phase: Tests First
```rust
#[test]
fn test_mining_near_edge_weakens_hull() {
    let mut app = App::new();
    app.add_systems(Update, process_hull_integrity);

    // Create an asteroid grid
    let mut grid = TerrainGrid::new(20, 20);
    grid.set(19, 10, TerrainType::AsteroidWall); // Edge wall
    grid.set(18, 10, TerrainType::AsteroidWall); // Inner wall
    app.world_mut().insert_resource(grid);
    app.world_mut().insert_resource(HullIntegrity { integrity: 100.0 });

    // Mine the inner wall (safely)
    app.world_mut().send_event(MineEvent { pos: GridPosition { x: 18, y: 10 } });
    app.update();

    // Integrity is fine
    assert_eq!(app.world().resource::<HullIntegrity>().integrity, 100.0);

    // Mine the edge wall (danger)
    app.world_mut().send_event(MineEvent { pos: GridPosition { x: 19, y: 10 } });
    app.update();

    // Integrity drops
    assert!(app.world().resource::<HullIntegrity>().integrity < 100.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_hull_integrity(
    mut events: EventReader<MineEvent>,
    mut integrity: ResMut<HullIntegrity>,
    grid: Res<TerrainGrid>,
) {
    for ev in events.read() {
        // If mining on the exact edge of the grid, it damages the hull
        if ev.pos.x == 0 || ev.pos.x == grid.width as i32 - 1 || ev.pos.y == 0 || ev.pos.y == grid.height as i32 - 1 {
            integrity.integrity -= 10.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an "edge proximity" formula instead of just checking the absolute edge, so mining *near* the edge is also dangerous.
- Tie Hull Integrity failure to a catastrophic venting event.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure events are processed correctly in the update schedule.

## 8. Questions
*Builder: add questions here if spec is unclear.*
