# 1112 - Tectonic Fracking

## 1. Overview
Squeezing the planet until it screams. Injecting "Liquid Waste" into deep crustal faults forces out pockets of resources. High efficiency waste disposal + resource gain, but increases "Seismic Instability" rapidly.

## 2. Dependencies
- Waste Management
- Resource Generation
- Seismic Activity

## 3. RED Phase: Tests First
```rust
#[test]
fn test_fracking_consumes_waste_generates_resources() {
    let mut app = App::new();
    app.add_plugins((WastePlugin, ResourcePlugin));

    // Arrange
    let fracker = app.world_mut().spawn(TectonicFracker).id();
    app.world_mut().insert_resource(ColonyWaste { amount: 100.0 });

    // Act
    app.world_mut().send_event(FrackEvent { entity: fracker });
    app.update();

    // Assert
    let waste = app.world().resource::<ColonyWaste>();
    assert!(waste.amount < 100.0);
    // Check some resource was generated (e.g. NaturalGas)
}

#[test]
fn test_fracking_increases_seismic_instability() {
    let mut app = App::new();
    app.add_plugins(SeismicPlugin);

    let fracker = app.world_mut().spawn(TectonicFracker).id();
    app.world_mut().insert_resource(SeismicInstability { level: 0.0 });

    // Act
    app.world_mut().send_event(FrackEvent { entity: fracker });
    app.update();

    // Assert
    let instability = app.world().resource::<SeismicInstability>();
    assert!(instability.level > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct TectonicFracker;
// systems...
```

## 5. REFACTOR Phase: Quality & Design
- Balance instability gain.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Tie instability to existing event triggers.

## 8. Questions
*Builder: add questions here if spec is unclear.*
