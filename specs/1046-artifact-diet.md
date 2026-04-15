# 1046 - The Artifact Diet

## 1. Overview
**Layer:** 1
**Fantasy:** Desperate colonists start eating strange alien relics out of desperation, with horrifying and miraculous long-term effects.
**Mechanic:** During severe famines, pops can optionally consume raw, unrefined "Precursor Artifacts" if no food is available. This immediately satisfies hunger but slowly mutates their genetic traits or imparts latent psionics over several generations.
**Emergence:** Your colony survives a massive early famine by eating artifacts. Generations later, the descendants of those survivors are fundamentally alien and require completely different environmental conditions to thrive, causing a physical schism in your population.
**Tension:** Do you let your colonists starve, or let them eat the artifacts and accept the unknown consequences down the line?

## 2. Dependencies
- **Layer 1 Needs (005):** Pops must have the `Hunger` component.
- **Layer 1 Ruins/Artifacts (913):** Must have the concept of `Artifact` items or resources.
- **Layer 1 Genetics/Traits:** Must be able to apply trait mutations to pops.

## 3. RED Phase: Tests First

```rust
// scale/src/layer1/needs/artifact_diet_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{Hunger, NeedStatus};
    use crate::layer1::culture::artifacts::Artifact;
    use crate::layer1::population::{Pop, TraitMutation};

    #[test]
    fn test_starving_pop_consumes_artifact() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, consume_artifacts_during_famine_system);

        let pop_entity = app.world_mut().spawn((
            Pop::default(),
            Hunger { value: 100.0 }, // Starving
        )).id();

        let artifact_entity = app.world_mut().spawn(Artifact {
            is_edible_in_emergency: true,
            ..Default::default()
        }).id();

        // Act
        app.update();

        // Assert
        let hunger = app.world().get::<Hunger>(pop_entity).unwrap();
        assert!(hunger.value < 10.0, "Hunger should be satisfied by the artifact");
        assert!(app.world().get_entity(artifact_entity).is_none(), "Artifact should be consumed");

        // Ensure mutation was queued
        assert!(app.world().get::<TraitMutation>(pop_entity).is_some(), "Consuming an artifact should cause a trait mutation");
    }

    #[test]
    fn test_well_fed_pop_ignores_artifact() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, consume_artifacts_during_famine_system);

        let pop_entity = app.world_mut().spawn((
            Pop::default(),
            Hunger { value: 0.0 }, // Well-fed
        )).id();

        let artifact_entity = app.world_mut().spawn(Artifact {
            is_edible_in_emergency: true,
            ..Default::default()
        }).id();

        // Act
        app.update();

        // Assert
        let hunger = app.world().get::<Hunger>(pop_entity).unwrap();
        assert_eq!(hunger.value, 0.0, "Hunger should remain unchanged");
        assert!(app.world().get_entity(artifact_entity).is_some(), "Artifact should NOT be consumed if pop is not starving");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// scale/src/layer1/needs/artifact_diet.rs
use bevy::prelude::*;
use crate::layer1::needs::Hunger;
use crate::layer1::culture::artifacts::Artifact;
use crate::layer1::population::{Pop, TraitMutation};

pub fn consume_artifacts_during_famine_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Hunger), With<Pop>>,
    artifacts: Query<(Entity, &Artifact)>,
) {
    for (pop_entity, mut hunger) in pops.iter_mut() {
        // Threshold for severe famine
        if hunger.value >= 90.0 {
            for (artifact_entity, artifact) in artifacts.iter() {
                if artifact.is_edible_in_emergency {
                    // Consume artifact
                    commands.entity(artifact_entity).despawn();

                    // Satisfy hunger
                    hunger.value = 0.0;

                    // Apply mutation
                    commands.entity(pop_entity).insert(TraitMutation {
                        mutation_type: "Precursor Resonance".to_string(),
                        severity: 0.5,
                    });

                    break; // One artifact is enough
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding/Logistics:** Pops shouldn't consume artifacts from across the map instantly. Integrate with hauling and job tasks so they must walk to the artifact.
- **Player Edict:** Add a colony edict to "Forbid Eating Artifacts", allowing players to choose mass starvation over genetic mutation.
- **Mutation Variety:** `TraitMutation` should be expanded to a list of potential random genetic drift effects rather than a hardcoded string.
- **Pacing:** The mutation might manifest silently at first, only appearing on the pop's children.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] A starving pop will consume an artifact if no normal food is available, reducing hunger to 0.
- [ ] Eating an artifact applies a `TraitMutation` component to the pop.

## 7. Technical Guidance
- **Integration Points:** Add the `consume_artifacts_during_famine_system` to `src/layer1/needs.rs` execution pipeline, specifically checking it after standard food consumption logic fails.
- **Artifact Component:** You'll likely need to modify the `Artifact` component to include a tag like `is_edible_in_emergency` or rely on a `EdibleAnomaly` component to mark which artifacts can be eaten.
- **Performance:** Iterating over all artifacts for every starving pop is O(N*M). If artifacts are numerous, this needs a spatial query or a centralized stockpile check.

## 8. Questions
*Builder: add questions here if spec is unclear.*
