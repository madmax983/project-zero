# 1215: The "Empty" Room

## 1. Overview
**Layer:** 1

**Fantasy:** In a crowded station, space is the ultimate luxury. The silence of a cathedral.

**Mechanic:** Designating a "Sanctuary" zone requires it to be *empty* (no furniture/machines). Pops visit to reduce Stress. If it becomes cluttered (even by a single item), the bonus vanishes.

**Emergence:** Pops start leaving "offerings" (flowers, rocks) in the Sanctuary, accidentally ruining its feng shui. You have to be the janitor of the void.

**Tension:** Utility (Storage/Housing) vs. Negative Space (Mental Health).

## 2. Dependencies
- Zones / Designations
- Clutter/Item system
- Unrest / Stress system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_sanctuary_reduces_stress_when_empty() {
    let mut app = App::new();
    app.add_systems(Update, evaluate_sanctuary_stress);

    // Create an empty sanctuary zone
    let mut zone_grid = ZoneGrid::new(10, 10);
    zone_grid.set(5, 5, ZoneType::Sanctuary);
    app.world_mut().insert_resource(zone_grid);
    app.world_mut().insert_resource(ClutterGrid::new(10, 10)); // Empty clutter

    // Pop in sanctuary
    let pop = app.world_mut().spawn((
        Pop,
        GridPosition { x: 5, y: 5 },
        CabinFever { stress: 50.0 }, // Has stress
    )).id();

    app.update();

    let fever = app.world().get::<CabinFever>(pop).unwrap();
    assert!(fever.stress < 50.0); // Stress reduced
}

#[test]
fn test_sanctuary_bonus_vanishes_when_cluttered() {
    let mut app = App::new();
    app.add_systems(Update, evaluate_sanctuary_stress);

    // Create a sanctuary zone
    let mut zone_grid = ZoneGrid::new(10, 10);
    zone_grid.set(5, 5, ZoneType::Sanctuary);
    app.world_mut().insert_resource(zone_grid);

    // ADD CLUTTER
    let mut clutter_grid = ClutterGrid::new(10, 10);
    clutter_grid.add(5, 5, 10.0);
    app.world_mut().insert_resource(clutter_grid);

    // Pop in sanctuary
    let pop = app.world_mut().spawn((
        Pop,
        GridPosition { x: 5, y: 5 },
        CabinFever { stress: 50.0 },
    )).id();

    app.update();

    let fever = app.world().get::<CabinFever>(pop).unwrap();
    assert_eq!(fever.stress, 50.0); // Stress NOT reduced because of clutter
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn evaluate_sanctuary_stress(
    mut query: Query<(&GridPosition, &mut CabinFever), With<Pop>>,
    zone_grid: Res<ZoneGrid>,
    clutter_grid: Res<ClutterGrid>,
) {
    for (pos, mut fever) in query.iter_mut() {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Sanctuary {
            if clutter_grid.get(pos.x, pos.y) == 0.0 {
                // It's empty, grant the bonus!
                fever.stress -= 1.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Hook into the Utility AI so Pops actively seek out empty Sanctuaries when their stress is high.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure the `ZoneType::Sanctuary` is properly defined and registered in the Utility AI buffers.

## 8. Questions
*Builder: add questions here if spec is unclear.*
