# 1185: Keystone Species

## 1. Overview
**Layer:** 1

**Fantasy:** Pulling one thread unravels the sweater. The ecosystem relies on a single pillar.

**Mechanic:** A specific flora/fauna supports the rest of the biome (e.g., "Water-Retaining Cactus" providing moisture for grazers). Killing it causes the biome to collapse into desert/wasteland, killing dependent species.

**Emergence:** You harvest the "useless" cactus for water. The local "Grazer-Beasts" die of thirst. The "Hunter-Beasts" get hungry and attack your livestock. The forest turns to dust.

**Tension:** Resource exploitation vs. Ecological stability.

---

## 2. Dependencies
- Base ECS system
- Event bus

## 3. RED Phase: Tests First
```rust
#[test]
fn test_keystone_species_bonus() {
    // Arrange: Create a world and spawn a keystone species pop
    let mut app = App::new();
    app.insert_resource(PlanetaryBonus { value: 0 });
    app.add_systems(Update, apply_keystone_bonus);
    let pop_entity = app.world_mut().spawn((Pop, KeystoneSpecies)).id();

    // Act: Run the simulation
    app.update();

    // Assert: Verify the keystone pop provides a planetary bonus
    let bonus = app.world().resource::<PlanetaryBonus>();
    assert!(bonus.value > 0);
}

#[test]
fn test_keystone_species_removal() {
    // Ensure bonus is removed when keystone species pop dies
    let mut app = App::new();
    app.insert_resource(PlanetaryBonus { value: 0 });
    app.add_systems(Update, apply_keystone_bonus);
    let pop_entity = app.world_mut().spawn((Pop, KeystoneSpecies)).id();
    app.update();
    app.world_mut().entity_mut(pop_entity).despawn();
    app.update();
    let bonus = app.world().resource::<PlanetaryBonus>();
    assert_eq!(bonus.value, 0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The simplest code to make the tests pass
fn apply_keystone_bonus(
    query: Query<&KeystoneSpecies, With<Pop>>,
    mut bonus: ResMut<PlanetaryBonus>,
) {
    if !query.is_empty() {
        bonus.value = 10;
    } else {
        bonus.value = 0;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Refactor resource mutation to an event-based system if PlanetaryBonus needs to be observed by other systems.

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
