# 276: Atmospheric Feedback

## 1. Overview

**Layer:** Cross-layer

**Fantasy:** The planet reacts to your industry. You are not just building on the map; you are changing the map.

**Mechanic:** Heavy industry generates "Smog". Smog reduces solar power and happiness but increases "Industrial Gloom". Trees absorb Smog. Pollution levels affect Layer 2 planet stats (habitability).

## 2. Dependencies
- 063 (Atmospheric Simulation), 042 (Energy System), 031 (Pop Morale)

## 3. RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_heavy_industry_generates_smog() {
    let mut world = App::new();
    let factory = world.world_mut().spawn((Building, HeavyIndustry, Active)).id();

    // Tick simulation
    world.update();

    // Assert smog generated
    let smog_level = world.world().resource::<Atmosphere>().smog_level;
    assert!(smog_level > 0.0);
}

#[test]
fn test_smog_reduces_solar_power() {
    let mut world = App::new();
    world.insert_resource(Atmosphere { smog_level: 0.5 }); // 50% smog
    let solar_panel = world.world_mut().spawn((Building, SolarPanel { base_output: 10.0 })).id();

    // Tick power generation
    world.update();

    // Assert output is reduced
    let output = world.world().get::<PowerOutput>(solar_panel).unwrap().current;
    assert_eq!(output, 5.0); // 50% reduction
}

#[test]
fn test_trees_absorb_smog() {
    let mut world = App::new();
    world.insert_resource(Atmosphere { smog_level: 10.0 });
    world.world_mut().spawn((Tree, Flora));

    // Tick simulation
    world.update();

    // Assert smog is reduced
    let smog_level = world.world().resource::<Atmosphere>().smog_level;
    assert!(smog_level < 10.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- Identify refactoring opportunities
- Extract magic numbers into `layer1::constants` or appropriate configuration
- Improve system performance using optimized queries
- Improve API for better modularity

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Implement feature in a separate module.
- Register the system in the `ScheduleBuilder` or Bevy App.
- Use the correct event dispatch patterns if interacting with ECS.
- Ensure that we isolate complex logic into helper functions instead of large systems.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
