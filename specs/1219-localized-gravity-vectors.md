# 1219: Localized Gravity Vectors

## 1. Overview
**Layer:** 1

**Fantasy:** Walking on the ceiling. The station is an Escher painting.

**Mechanic:** "Gravity Plates" define "Down" for adjacent tiles. You can build rooms on walls or ceilings. Pops transition orientation when walking over curved plates. Zero-G zones allow floating.

**Emergence:** You build a high-density housing block on the ceiling of the hangar to save floor space. A power failure kills the gravity plates, and 500 people fall into the parked starships below.

**Tension:** Density (use all surfaces) vs. Complexity/Disorientation risk.

## 2. Dependencies
- Gravity system
- Map / Pathfinding system
- Power system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_unpowered_gravity_plates_cause_falling() {
    let mut app = App::new();
    app.add_systems(Update, process_localized_gravity);

    // Create a pop on the ceiling
    let pop = app.world_mut().spawn((
        Pop,
        GridPosition { x: 5, y: 0, z: 10 }, // z=10 is ceiling
        Orientation { vector: Vec3::new(0.0, 0.0, -1.0) }, // "Down" is towards z=0
    )).id();

    // Create the gravity plate powering the ceiling
    let plate = app.world_mut().spawn((
        Building { type_: BuildingType::GravityPlate },
        Powered { is_powered: false }, // Unpowered!
        GridPosition { x: 5, y: 0, z: 10 },
    )).id();

    app.update();

    // Pop should be assigned a "Falling" component due to zero-G/unpowered state
    assert!(app.world().get::<Falling>(pop).is_some());
}

#[test]
fn test_powered_gravity_plates_maintain_orientation() {
    let mut app = App::new();
    app.add_systems(Update, process_localized_gravity);

    // Create a pop on the ceiling
    let pop = app.world_mut().spawn((
        Pop,
        GridPosition { x: 5, y: 0, z: 10 },
        Orientation { vector: Vec3::new(0.0, 0.0, -1.0) },
    )).id();

    // Create the gravity plate powering the ceiling
    let plate = app.world_mut().spawn((
        Building { type_: BuildingType::GravityPlate },
        Powered { is_powered: true }, // Powered!
        GridPosition { x: 5, y: 0, z: 10 },
    )).id();

    app.update();

    // Pop should NOT be falling
    assert!(app.world().get::<Falling>(pop).is_none());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_localized_gravity(
    mut commands: Commands,
    pop_query: Query<(Entity, &GridPosition), With<Pop>>,
    plate_query: Query<(&GridPosition, &Powered), With<Building>>,
) {
    for (pop_entity, pop_pos) in pop_query.iter() {
        let mut has_gravity = false;

        // Find if there is a powered plate at this position
        for (plate_pos, powered) in plate_query.iter() {
            if pop_pos == plate_pos && powered.is_powered {
                has_gravity = true;
                break;
            }
        }

        if !has_gravity {
            commands.entity(pop_entity).insert(Falling);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an actual "falling" logic that moves the pop down Z-levels and deals damage upon impact.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure events are processed correctly in the update schedule.

## 8. Questions
*Builder: add questions here if spec is unclear.*
