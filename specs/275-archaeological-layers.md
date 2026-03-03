# 275: Archaeological Layers

## 1. Overview

**Layer:** 1

**Fantasy:** The ground beneath your feet is a graveyard of civilizations. You are digging on a world that has seen empires rise and fall before.

**Mechanic:** Deep terrain layers contain "Ruins" tiles. Excavating them yields Artifacts (lore/resources) but risks "Old World Maladies" (curses/diseases).

## 2. Dependencies
- 018 (Mining), 156 (Xeno-Artifacts)

## 3. RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_excavate_ruins_yields_artifact() {
    let mut world = App::new();
    let miner = world.world_mut().spawn(Pop).id();
    let ruins_tile = world.world_mut().spawn((Tile, Ruins { difficulty: 10.0 })).id();

    // Simulate completing excavation
    complete_excavation(&mut world, miner, ruins_tile);

    // Assert artifact is spawned
    let artifacts: Vec<_> = world.world_mut().query::<&Artifact>().iter(&world.world()).collect();
    assert_eq!(artifacts.len(), 1);
}

#[test]
fn test_excavate_ruins_triggers_malady() {
    let mut world = App::new();
    // Setup rigged ruins that guarantees malady
    let miner = world.world_mut().spawn(Pop).id();
    let ruins_tile = world.world_mut().spawn((Tile, Ruins { difficulty: 10.0, malady_chance: 1.0 })).id();

    complete_excavation(&mut world, miner, ruins_tile);

    // Assert miner gets malady
    assert!(world.world().get::<Malady>(miner).is_some());
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
