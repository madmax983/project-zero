# 278: Acoustic Zones

## 1. Overview

**Layer:** 1

**Fantasy:** The colony is noisy. Living next to a stamping mill is miserable, but living next to a park is peaceful.

**Mechanic:** Machines emit "Noise" that spreads across the grid (attenuating over distance). High noise reduces Sleep quality and increases Stress. Walls block noise.

## 2. Dependencies
- 060 (Acoustic Simulation), 006 (Building Placement), 005 (Pop Needs)

## 3. RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_machine_emits_noise() {
    let mut world = App::new();
    let machine = world.world_mut().spawn((Building, NoiseEmitter { volume: 10.0 }, GridPosition::new(0, 0))).id();

    // Update acoustics
    world.update();

    // Assert noise map has noise
    let noise_map = world.world().resource::<NoiseMap>();
    assert!(noise_map.get(GridPosition::new(0, 0)) > 0.0);
    assert!(noise_map.get(GridPosition::new(1, 0)) > 0.0); // Spreads
}

#[test]
fn test_noise_reduces_sleep_quality() {
    let mut world = App::new();
    world.insert_resource(NoiseMap::new_with_ambient(5.0)); // High ambient noise
    let pop = world.world_mut().spawn((Pop, Needs::default(), Sleeping)).id();

    // Tick metabolism
    world.update();

    // Assert rest gained is less than optimal
    let needs = world.world().get::<Needs>(pop).unwrap();
    assert!(needs.rest < Needs::default().rest + OPTIMAL_SLEEP_GAIN);
}

#[test]
fn test_walls_block_noise() {
    let mut world = App::new();
    // Machine at 0,0. Wall at 1,0.
    world.world_mut().spawn((Building, NoiseEmitter { volume: 10.0 }, GridPosition::new(0, 0)));
    world.world_mut().spawn((Building, Wall, GridPosition::new(1, 0)));

    world.update();

    let noise_map = world.world().resource::<NoiseMap>();
    assert!(noise_map.get(GridPosition::new(2, 0)) < noise_map.get(GridPosition::new(0, 1))); // Attenuated by wall
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
