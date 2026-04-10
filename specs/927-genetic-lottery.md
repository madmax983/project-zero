# 927 - The Genetic Lottery

## 1. Overview

**Layer:** Cross-layer
**Fantasy:** A random genetic mutation in one generation dictates the future of your civilization.
**Mechanic:** Every new generation of Pops has a tiny chance to develop a spontaneous, radical genetic trait (e.g., zero-G affinity, extreme radiation tolerance, or dietary obligate cannibalism).
**Emergence:** A single pop is born with the ability to survive in hard vacuum. Centuries later, their descendants form an elite, unkillable void-navy that refuses to land on planets, creating a bifurcated civilization of "grounders" and "void-born."
**Tension:** Culling or isolating anomalous pops early to preserve a unified human baseline, versus letting mutations spread and radically altering your empire's future capabilities and culture.

## 2. Dependencies

- Pop Lifecycle / Reproduction Systems
- Trait / Genetics tracking for Pops
- Modifier application systems (e.g. for environment tolerance)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct GeneticTraitTracker { mutation_chance: f32 }

    #[derive(Component)]
    struct PopGenome { traits: Vec<String> }

    #[derive(Event)]
    struct PopBornEvent { parent: Entity, child: Entity }

    #[test]
    fn test_pop_inherits_and_mutates_traits() {
        let mut app = App::new();
        app.add_event::<PopBornEvent>();
        app.add_systems(Update, process_pop_birth_mutation_system);

        let parent = app.world_mut().spawn((
            PopGenome { traits: vec!["Standard".to_string()] },
            GeneticTraitTracker { mutation_chance: 1.0 }, // 100% chance for test
        )).id();

        let child = app.world_mut().spawn_empty().id();

        app.world_mut().send_event(PopBornEvent { parent, child });
        app.update();

        let child_genome = app.world().get::<PopGenome>(child).unwrap();
        // Child should have inherited traits and gained a mutation
        assert!(child_genome.traits.contains(&"Standard".to_string()));
        assert!(child_genome.traits.len() > 1); // Mutated trait added
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn process_pop_birth_mutation_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Create an Enum or Registry for specific genetic mutations rather than using raw Strings, allowing them to hook into existing gameplay modifiers easily.
- Abstract the mutation logic so environmental factors (like radiation) can dynamically adjust the `mutation_chance`.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops inherit genetic traits from parents during birth, with a configurable chance for spontaneous mutation.

## 7. Technical Guidance

- Integrate with the existing `PopBornEvent` or equivalent reproduction trigger.
- Keep the baseline mutation chance extremely low in actual gameplay to maintain the rarity and significance of the "Genetic Lottery".

## 8. Questions
*Builder: add questions here if spec is unclear.*
