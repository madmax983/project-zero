# 1111 - The Void Stare

## 1. Overview
If you gaze long into an abyss, the abyss also gazes into you. Windows/Observatories facing "Empty Space" increase Stress/Insanity over time. Views of "Life" reduce it.

## 2. Dependencies
- Pop Health & Morale
- Building Placement

## 3. RED Phase: Tests First
```rust
#[test]
fn test_void_facing_window_increases_stress() {
    let mut app = App::new();
    app.add_plugins(MoralePlugin);

    // Arrange: Pop in an observatory facing void
    let pop = app.world_mut().spawn((Pop::new(), Stress { level: 10.0 }, VoidStareEffect { facing_void: true })).id();

    // Act: Advance simulation
    app.update();

    // Assert stress increase
    let stress = app.world().get::<Stress>(pop).unwrap();
    assert!(stress.level > 10.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct VoidStareEffect { pub facing_void: bool }
// systems...
```

## 5. REFACTOR Phase: Quality & Design
- Add raycasting for view calculation.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Raycast from building to check for celestial bodies.

## 8. Questions
*Builder: add questions here if spec is unclear.*
