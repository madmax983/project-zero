# 1227: Selective Amnesia

## 1. Overview
**Layer:** 1

**Fantasy:** Eternal Sunshine of the Spotless Mind.

**Mechanic:** "Memory Wiping" medical procedure. Removes "Trauma" traits/memories. However, it also deletes linked Skills or Relationships.

**Emergence:** You wipe the PTSD of your best soldier so he can fight again. He forgets his wife (also a soldier), and she leaves him, causing a depression spiral.

**Tension:** Mental Health vs. Identity/Skills.

## 2. Dependencies
- Medical system
- Pop Memory/Trait system
- Pop Relations

## 3. RED Phase: Tests First
```rust
#[test]
fn test_memory_wipe_removes_trauma_and_relationships() {
    let mut app = App::new();
    app.add_systems(Update, process_memory_wipe_procedure);

    // Spawn a pop with trauma and a spouse
    let pop = app.world_mut().spawn((
        Pop,
        Memories { traumas: vec!["War".to_string()] },
        Relationships { spouse: Some(Entity::PLACEHOLDER) },
        MemoryWipeProcedure { active: true },
    )).id();

    app.update();

    let memories = app.world().get::<Memories>(pop).unwrap();
    let relationships = app.world().get::<Relationships>(pop).unwrap();

    // Trauma is gone
    assert!(memories.traumas.is_empty());
    // Spouse is forgotten!
    assert!(relationships.spouse.is_none());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_memory_wipe_procedure(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Memories, &mut Relationships, &MemoryWipeProcedure)>,
) {
    for (entity, mut memories, mut relations, procedure) in query.iter_mut() {
        if procedure.active {
            // Wipe trauma
            memories.traumas.clear();
            // Wipe relations (collateral damage)
            relations.spouse = None;

            // Procedure complete
            commands.entity(entity).remove::<MemoryWipeProcedure>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Trigger a `RelationshipBrokenEvent` so the spouse's morale actually drops in response.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Hook into the `src/layer1/memory.rs` and `src/layer1/social.rs` systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*
