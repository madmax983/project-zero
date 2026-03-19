
# Specification: Xeno-Artifacts

## 1. Overview
**Layer:** 1
**Fantasy:** Stumbling upon something ancient, powerful, and utterly incomprehensible. The "Monolith" experience.
**Mechanic:** Map generation includes rare, indestructible "Artifact" tiles. They emit auras (e.g., "Insight": +Science XP, +Stress; "Vitality": +Heal rate, +Hunger). They cannot be moved, only built around.
**Emergence:** Your research lab is built around a "Humming Obelisk." The scientists are brilliant but slowly going insane. A cult forms demanding you "Feed the Stone."
**Tension:** Exploit the artifact (bonuses) vs. Quarantine it (safety).

## 2. Dependencies
- Map generation system (`src/layer1/terrain.rs` or `src/layer1/map.rs`)
- Aura/Modifier system for entities (`src/layer1/needs.rs` or similar)
- Basic Pop mechanics

## 3. RED Phase: Tests First

```rust
use bevy_ecs::prelude::*;
// Assume generic test world setup exists
// use crate::tests::setup_test_world;

#[test]
fn test_artifact_spawns_during_map_generation() {
    let mut world = World::new();
    // Simulate map generation with artifacts enabled
    // generate_terrain(&mut world, MapConfig { allow_artifacts: true });

    // Act
    // (mocking generation)
    world.spawn((TerrainType::Artifact, GridPosition { x: 5, y: 5 }));

    // Assert: At least one tile has the TerrainType::Artifact
    let mut query = world.query::<&TerrainType>();
    let artifact_count = query.iter(&world).filter(|&t| *t == TerrainType::Artifact).count();
    assert!(artifact_count > 0, "Map generation should spawn at least one Artifact.");
}

#[test]
fn test_artifact_emits_aura_to_nearby_pops() {
    let mut world = World::new();
    // Arrange: Spawn an Artifact entity with a specific aura (e.g., Insight: +Science, +Stress)
    let artifact = world.spawn((
        TerrainType::Artifact,
        GridPosition { x: 10, y: 10 },
        ArtifactAura { radius: 3, effect_type: AuraEffect::Insight }
    )).id();

    // Spawn a Pop within the aura's radius
    let pop = world.spawn((
        Pop,
        GridPosition { x: 11, y: 10 },
        Mood::default(),
        Stress::default()
    )).id();

    // Act: Advance simulation by several ticks
    // apply_artifact_auras_system(&mut world);

    // Assert: The Pop receives the specified modifiers (e.g., increased Stress)
    let stress = world.get::<Stress>(pop).unwrap();
    assert!(stress.value > 0.0, "Pop within Artifact aura should receive increased stress.");
}

#[test]
fn test_artifact_indestructible() {
    let mut world = World::new();
    // Arrange: Spawn an Artifact on the map
    let artifact = world.spawn((
        TerrainType::Artifact,
        GridPosition { x: 5, y: 5 }
    )).id();

    // Act: Attempt to issue a 'Mine' or 'Destroy' command on the Artifact's tile
    // let result = execute_mine_action(&mut world, artifact);

    // Assert: The command is rejected or the entity remains intact after the action completes
    // assert!(result.is_err(), "Artifacts should be indestructible.");
    assert!(world.get_entity(artifact).is_ok(), "Artifact entity should still exist.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// 1. Add `Artifact` and `ArtifactAura` components to define the artifact's properties and area of effect.
// 2. Modify map generation to randomly spawn a small number of `Artifact` entities on the grid.
// 3. Create a system `apply_artifact_auras_system` that queries all `ArtifactAura` components and applies their effects (e.g., `MoodModifier`) to Pops within range.
// 4. Ensure `TerrainType` or the mining validation logic specifically prevents targeting `Artifact` tiles.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure the aura system is generic enough to be reused for other environmental effects (e.g., heaters, loudspeakers).
- Consider caching spatial queries for pops within aura radii if performance becomes an issue on larger maps.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Artifacts generate on the map and cannot be destroyed.
- [ ] Artifacts apply their defined auras to nearby Pops.

## 7. Technical Guidance
- The aura effect should probably be a periodic pulse or a zone-based modifier that gets added/removed as Pops enter/leave the radius, rather than accumulating infinitely.
- Auras that increase Stress should have a cap to prevent instant mental breakdowns.

## 8. Questions
*Builder: add questions here if spec is unclear.*
