# 1191: Keystone Species

**1. Overview**
In Layer 1 of SCALE, a planet's ecosystem is fragile. Pulling one thread can unravel the entire sweater. This feature introduces the concept of a "Keystone Species"—a specific flora or fauna that supports the rest of the biome. If the player harvests, hunts, or destroys this pillar species, the biome collapses into a wasteland, causing all dependent species to die off or flee.

This mechanic introduces a tension between short-term resource exploitation and long-term ecological stability.

**2. Dependencies**
- Base Layer 1 grid and biome/terrain components.
- Flora and Fauna entities (`layer1::ecology::Flora`, `layer1::ecology::Fauna`).
- Resource harvesting and gathering systems.

**3. RED Phase: Tests First**
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_biome_collapses_when_keystone_species_removed() {
    // Arrange: Create a biome tile with a Keystone component on a Flora entity.
    // Also spawn dependent Fauna on that tile.
    // Act: Remove/harvest the Keystone Flora entity and advance simulation time.
    // Assert: The biome tile transforms into a Wasteland/Desert state.
    // Assert: The dependent Fauna entities are despawned or marked as dying.
}

#[test]
fn test_harvesting_non_keystone_species_does_not_collapse_biome() {
    // Arrange: Create a biome tile with multiple flora, only one being Keystone.
    // Act: Harvest a non-Keystone flora entity.
    // Assert: The biome tile remains intact and dependent Fauna survive.
}

#[test]
fn test_keystone_collapse_cascades_to_adjacent_tiles_slowly() {
    // Arrange: Create a cluster of biome tiles. Remove the keystone species from the center tile.
    // Act: Advance simulation time significantly.
    // Assert: Adjacent tiles of the same biome type begin to lose their health/vitality.
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
// The SIMPLEST code that makes tests pass

use bevy::prelude::*;

#[derive(Component)]
pub struct KeystoneSpecies {
    pub biome_id: Entity, // The biome tile this species supports
}

#[derive(Component)]
pub struct BiomeDependency {
    pub biome_id: Entity,
}

#[derive(Component)]
pub struct BiomeTile {
    pub is_collapsed: bool,
}

// When a KeystoneSpecies is removed (e.g., via a harvest event or despawn),
// flag the associated BiomeTile as collapsed.
pub fn process_keystone_removal(
    mut removed_keystone: RemovedComponents<KeystoneSpecies>,
    // In a real implementation, we'd need a way to track which biome it belonged to,
    // perhaps by listening to a specific death/harvest event rather than just RemovedComponents,
    // or by storing the biome_id in a separate resource/event.
    // For this minimal pass, assume we have an event `KeystoneHarvestedEvent { biome_entity }`.
    mut events: EventReader<KeystoneHarvestedEvent>,
    mut biomes: Query<&mut BiomeTile>,
) {
    for event in events.read() {
        if let Ok(mut biome) = biomes.get_mut(event.biome_entity) {
            biome.is_collapsed = true;
        }
    }
}

pub fn handle_biome_collapse(
    biomes: Query<(Entity, &BiomeTile), Changed<BiomeTile>>,
    mut dependents: Query<(Entity, &BiomeDependency)>,
    mut commands: Commands,
) {
    for (biome_entity, biome) in biomes.iter() {
        if biome.is_collapsed {
            // Kill off dependent entities
            for (dep_entity, dep) in dependents.iter_mut() {
                if dep.biome_id == biome_entity {
                    commands.entity(dep_entity).despawn();
                }
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Instead of instantly despawning dependent fauna, consider giving them a `Starving` or `Thirsty` component so they slowly die off, giving the player a brief window to react (or just watch the consequences).
- The transition from a lush biome to a wasteland should change the underlying `GridPosition` terrain type, affecting movement costs and aesthetics.
- Implement a `KeystoneHarvestedEvent` to cleanly decouple the harvesting logic from the ecology logic.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Harvesting a keystone species triggers a biome collapse event.
- [ ] Dependent flora/fauna react to the collapse (die or migrate).

**7. Technical Guidance**
- Tie this into the existing `layer1::ecology` or `layer1::environment` systems.
- Be careful with `RemovedComponents` as you lose access to the data on the component. It is highly recommended to fire an explicit event when the entity is destroyed instead.
- Consider adding a visual indicator (like a unique shader or particle effect) to subtlely hint to the player that a specific entity is a keystone.

**8. Questions**
*Builder: Add questions here if integration with existing terrain/biome systems is unclear.*

*Architect:* For MVP, mapping a Keystone species to a specific Biome component is sufficient. Feel free to extend to specific terrain tags if necessary.
