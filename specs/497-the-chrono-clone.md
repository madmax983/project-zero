# Specification 497: The Chrono-Clone

## 1. Overview
A rare temporal anomaly or extreme endgame tech allows you to duplicate a specific Pop exactly as they were at a previous point in the simulation. The new "Chrono-Clone" has the original's old skills and memories.

## 2. Dependencies
- Memory System / Pop History Log
- Pop Generation/Cloning System
- Tech Tree (Endgame Tech)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_chrono_clone_creates_duplicate_with_past_stats() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Create an old snapshot of a pop with a level 5 skill
    let past_snapshot = PopSnapshot {
        original_id: Entity::from_raw(1),
        skills: vec![Skill { kind: SkillType::Mining, level: 5 }],
        memories: vec![],
    };

    // Act: Spawn a Chrono-Clone using the snapshot
    app.world_mut().send_event(SpawnChronoCloneEvent { snapshot: past_snapshot });
    app.update();

    // Assert: The new clone has the exact stats from the snapshot
    let clone_query = app.world_mut().query_filtered::<&Skills, With<ChronoClone>>().iter(app.world()).next();
    assert!(clone_query.is_some());
    let skills = clone_query.unwrap();
    assert_eq!(skills.get_level(SkillType::Mining), 5);
}

#[test]
fn test_chrono_clone_and_original_interact_differently() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Spawn an original pop with a recent trauma
    let original = app.world_mut().spawn((
        PopBundle::default(),
        MemoryLog { events: vec![MemoryEvent::WitnessedDeath] },
    )).id();

    // Spawn a clone from a snapshot BEFORE the trauma
    let clone = app.world_mut().spawn((
        PopBundle::default(),
        ChronoClone { source: original },
        MemoryLog { events: vec![] }, // No trauma
    )).id();

    // Act: Trigger an interaction that relies on memory
    app.world_mut().send_event(InteractWithGhostEvent { pop: original });
    app.world_mut().send_event(InteractWithGhostEvent { pop: clone });
    app.update();

    // Assert: The original panics, but the clone does not
    let orig_stress = app.world().get::<Stress>(original).unwrap().0;
    let clone_stress = app.world().get::<Stress>(clone).unwrap().0;

    assert!(orig_stress > 0.0);
    assert_eq!(clone_stress, 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// Add `SpawnChronoCloneEvent` and system to process it.
// Spawns a new entity copying components from the provided `PopSnapshot`.
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the Pop History Log so it doesn't store excessive snapshots per Pop.
- Ensure the duplication event handles relationships properly (the clone shouldn't automatically share all current relationships).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Cloning successfully revives a past version of a Pop with accurate historical state.

## 7. Technical Guidance
- Implement a periodic snapshotting mechanism or rely on major life events to store `PopSnapshot`s.
- The `ChronoCloneEvent` takes a `PopSnapshot` and spawns a new Entity using that data.
- The newly spawned entity receives a `ChronoClone` tag/trait for specific behavioral overrides or unique event triggers.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
