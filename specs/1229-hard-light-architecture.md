# 1229: Hard-Light Architecture

## 1. Overview
**Layer:** 1

**Fantasy:** Walls made of light.

**Mechanic:** Projectors create physical walls/bridges. They are instant to toggle on/off. They consume power. Zero HP (pass-through) if power fails.

**Emergence:** You use Hard-Light dams to hold back a lava flow. A brownout flickers the dam for 0.1 seconds. The lava gets through.

**Tension:** Flexibility/Speed vs. Fragility (Power dependence).

## 2. Dependencies
- Building system
- Power Grid
- Pathfinding/Collision

## 3. RED Phase: Tests First
```rust
#[test]
fn test_hard_light_wall_is_impassable_when_powered() {
    let mut app = App::new();
    app.add_systems(Update, update_hard_light_collision);

    // Spawn powered projector
    let projector = app.world_mut().spawn((
        Building { type_: BuildingType::HardLightProjector },
        Powered { is_powered: true },
        GridPosition { x: 5, y: 5 },
        Collision { is_solid: false },
    )).id();

    app.update();

    // The collision should now be solid
    let collision = app.world().get::<Collision>(projector).unwrap();
    assert!(collision.is_solid);
}

#[test]
fn test_hard_light_wall_is_passable_when_unpowered() {
    let mut app = App::new();
    app.add_systems(Update, update_hard_light_collision);

    // Spawn UNpowered projector
    let projector = app.world_mut().spawn((
        Building { type_: BuildingType::HardLightProjector },
        Powered { is_powered: false },
        GridPosition { x: 5, y: 5 },
        Collision { is_solid: true }, // Starts solid for test
    )).id();

    app.update();

    // The collision should drop to passable
    let collision = app.world().get::<Collision>(projector).unwrap();
    assert!(!collision.is_solid);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn update_hard_light_collision(
    mut query: Query<(&Powered, &mut Collision), With<Building>>,
) {
    for (powered, mut collision) in query.iter_mut() {
        // In a real implementation we'd filter for BuildingType::HardLightProjector
        if powered.is_powered {
            collision.is_solid = true;
        } else {
            collision.is_solid = false;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Allow the projector to project the wall several tiles away (not just on its own tile) to create real risk if the projector is destroyed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure `Collision` component interacts correctly with pathfinding logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
