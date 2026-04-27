# 1206: Phantom Infrastructure

## 1. Overview
**Layer:** 1

**Fantasy:** The base has a history you've forgotten. The spaghetti code of city planning.

**Mechanic:** Pipes and Cables built in the early game become "Occluded" (invisible) under floors and walls over time. Deconstructing a wall has a risk of severing a forgotten "pass-through" line powering a distant sector.

**Emergence:** You renovate the cafeteria for better aesthetics. You accidentally cut the main power line to the Cryo-Bay on the other side of the base. 50 sleepers thaw out angry and confused.

**Tension:** Renovation (Clean layout) vs. Inertia (Fear of breaking legacy systems).

## 2. Dependencies
- Base ECS system
- Infrastructure and Power systems (for cables/pipes)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_phantom_infrastructure_occlusion() {
    let mut app = App::new();
    app.add_systems(Update, process_infrastructure_occlusion);

    // Create an old cable entity
    let cable_entity = app.world_mut().spawn((
        Infrastructure { type_: InfraType::Cable },
        Age { ticks: 10000 },
        Visibility::Visible,
    )).id();

    // Run system
    app.update();

    // Assert that the old cable became occluded
    let visibility = app.world().get::<Visibility>(cable_entity).unwrap();
    assert_eq!(*visibility, Visibility::Hidden);
}

#[test]
fn test_deconstruction_severs_phantom_line() {
    let mut app = App::new();
    app.add_systems(Update, process_wall_deconstruction);

    // Create an occluded cable sharing the same tile as a wall being deconstructed
    let pos = GridPosition { x: 5, y: 5 };
    let cable = app.world_mut().spawn((
        Infrastructure { type_: InfraType::Cable },
        Visibility::Hidden,
        pos,
    )).id();

    let wall = app.world_mut().spawn((
        Building { type_: BuildingType::Wall },
        DeconstructCommand,
        pos,
    )).id();

    // Run system
    app.update();

    // The wall is gone, and the phantom cable should be severed (destroyed or marked as broken)
    assert!(app.world().get::<Building>(wall).is_none());
    assert!(app.world().get::<Severed>(cable).is_some());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_infrastructure_occlusion(
    mut query: Query<(&mut Visibility, &Age), With<Infrastructure>>,
) {
    for (mut visibility, age) in query.iter_mut() {
        if age.ticks > 5000 {
            *visibility = Visibility::Hidden;
        }
    }
}

fn process_wall_deconstruction(
    mut commands: Commands,
    wall_query: Query<(Entity, &GridPosition), (With<Building>, With<DeconstructCommand>)>,
    infra_query: Query<(Entity, &GridPosition, &Visibility), With<Infrastructure>>,
) {
    for (wall_entity, wall_pos) in wall_query.iter() {
        commands.entity(wall_entity).despawn();

        for (infra_entity, infra_pos, visibility) in infra_query.iter() {
            if wall_pos == infra_pos && *visibility == Visibility::Hidden {
                commands.entity(infra_entity).insert(Severed);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Improve the definition of "Visibility::Hidden" to ensure it integrates correctly with the Ratatui renderer so players truly cannot see it.
- Ensure the `Severed` state properly interrupts the power or resource grid.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure events are processed correctly in the update schedule.
- Need to check if `Visibility` enum exists or implement a specific `Occluded` component.

## 8. Questions
*Builder: add questions here if spec is unclear.*
