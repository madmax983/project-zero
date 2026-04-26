# 1186: Ship Personalities

## 1. Overview
**Layer:** 2

**Fantasy:** "She's got it where it counts, kid." The ship is a character, not just a stat block.

**Mechanic:** Ships gain positive/negative "Quirks" based on their history. Surviving a battle at 1% HP grants "Lucky" (dodge chance). Running out of fuel grants "Fuel Hog" (consumption penalty). Quirks persist through refits.

**Emergence:** You refuse to scrap the "Old Betsy" hauler because she has the "Lucky" trait and survives every pirate raid, even though she costs 2x fuel and leaks radiation.

**Tension:** Efficiency (New ships) vs. Sentiment/Quirks (Old ships).

---

## 2. Dependencies
- Base ECS system
- Event bus

## 3. RED Phase: Tests First
```rust
#[test]
fn test_ship_personality_quirk() {
    // Arrange: Create a ship with a specific personality trait
    let mut app = App::new();
    app.add_systems(Update, process_ship_quirks);
    let ship_entity = app.world_mut().spawn((Ship, Personality { quirk: Quirks::Stubborn })).id();

    // Act: Issue a movement command and run simulation
    app.world_mut().entity_mut(ship_entity).insert(MoveCommand { target: Vec2::new(10.0, 10.0) });
    app.update();

    // Assert: The stubborn ship occasionally ignores commands
    let has_command = app.world().entity(ship_entity).contains::<MoveCommand>();
    // Expect the command to be randomly dropped (mock RNG for deterministic test)
    assert!(!has_command);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_ship_quirks(
    mut commands: Commands,
    query: Query<(Entity, &Personality, &MoveCommand), With<Ship>>,
) {
    for (entity, personality, command) in query.iter() {
        if personality.quirk == Quirks::Stubborn {
            // Simplest implementation: always drop the command for stubborn ships
            commands.entity(entity).remove::<MoveCommand>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Instead of directly removing MoveCommand, implement an interrupt or error state so the UI can reflect the stubbornness.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure events are processed correctly in the update schedule.
- Remember to explicitly register all new systems, events, and resources to the main game app.

## 8. Questions
*Builder: add questions here if spec is unclear.*
