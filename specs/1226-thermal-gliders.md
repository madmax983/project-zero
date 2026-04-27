# 1226: Thermal Gliders

## 1. Overview
**Layer:** 1

**Fantasy:** Riding the heat of industry.

**Mechanic:** "Glider Haulers" consume no fuel but require "Updrafts" from Heat sources (Furnaces, Vents) to gain altitude/speed. Cold zones ground them.

**Emergence:** Your logistics network relies on the heat from the smelters. When the smelters shut down for maintenance, the gliders are grounded, and the supply chain collapses.

**Tension:** Centralized heat (Glider highways) vs. Dispersed cooling (Safety).

## 2. Dependencies
- Temperature/Heat grid
- Drone/Hauler movement logic

## 3. RED Phase: Tests First
```rust
#[test]
fn test_thermal_glider_needs_heat_to_move() {
    let mut app = App::new();
    app.add_systems(Update, process_thermal_glider_movement);

    // Create a cold temperature grid
    let mut temp_grid = TemperatureGrid::new(10, 10);
    temp_grid.set_temp(5, 5, 0.0); // Cold
    app.world_mut().insert_resource(temp_grid);

    // Spawn a glider trying to move
    let glider = app.world_mut().spawn((
        Glider,
        GridPosition { x: 5, y: 5 },
        MoveCommand { target: Vec2::new(6.0, 5.0) },
    )).id();

    app.update();

    // The glider should NOT have moved, it's grounded
    let pos = app.world().get::<GridPosition>(glider).unwrap();
    assert_eq!(pos.x, 5);
}

#[test]
fn test_thermal_glider_moves_in_heat() {
    let mut app = App::new();
    app.add_systems(Update, process_thermal_glider_movement);

    // Create a hot temperature grid
    let mut temp_grid = TemperatureGrid::new(10, 10);
    temp_grid.set_temp(5, 5, 100.0); // Hot! Updraft!
    app.world_mut().insert_resource(temp_grid);

    // Spawn a glider trying to move
    let glider = app.world_mut().spawn((
        Glider,
        GridPosition { x: 5, y: 5 },
        MoveCommand { target: Vec2::new(6.0, 5.0) },
    )).id();

    app.update();

    // The glider SHOULD have moved
    let pos = app.world().get::<GridPosition>(glider).unwrap();
    assert_eq!(pos.x, 6);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_thermal_glider_movement(
    mut query: Query<(&mut GridPosition, &MoveCommand), With<Glider>>,
    temp_grid: Res<TemperatureGrid>,
) {
    for (mut pos, cmd) in query.iter_mut() {
        let current_temp = temp_grid.get_temp(pos.x, pos.y);

        if current_temp > 50.0 {
            // Hot enough for updraft, allow movement (simplified teleport for GREEN phase)
            pos.x = cmd.target.x as i32;
            pos.y = cmd.target.y as i32;
        }
        // If cold, do nothing (grounded)
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Hook this into the pathfinding A* algorithm so gliders explicitly path through hot tiles rather than taking the shortest physical route.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Integrate into `src/layer1/movement.rs` or `src/layer1/logistics.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
