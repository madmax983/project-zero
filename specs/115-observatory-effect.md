# 115: The Overview Effect

## 1. Overview

Introduces the **Observatory** building and the **Observe** action.
- **Observatory**: A specialized building where pops study the cosmos. Requires `Tech::Astronomy`.
- **Observe Action**: Pops working here generate **Knowledge** (similar to Libraries) but also experience the "Overview Effect".
- **The Effect**: Observing the vastness of space triggers a temporary Mood Modifier. It can be **Positive** ("Inspired", +Mood) or **Negative** ("Existential Dread", -Mood), representing the dual nature of cosmic perspective.

This feature adds flavor to the late-game research loop and connects Layer 1 (Colony) to the theme of Layer 2 (Space) emotionally.

## 2. Dependencies

- `specs/007-building-housing.md` — Building System
- `specs/016-utility-ai-system.md` — Utility AI (Actions)
- `specs/029-knowledge-system.md` — Tech Tree & Knowledge Resource
- `specs/031-pop-morale.md` — Morale System

## 3. RED Phase: Tests First

Write these tests in `src/layer1/observatory_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::tech::{Tech, TechState};
    use crate::layer1::building::BuildingType;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::morale::{Morale, MoraleModifier};
    use crate::layer1::pop::{Pop, GridPosition};
    use crate::layer1::utility_ai::{ActionType, AssignmentType, AssignedTo};
    use crate::layer1::observatory::{Observatory, process_observe_system};

    #[test]
    fn test_astronomy_tech_exists() {
        // Just verifying variant exists
        let _ = Tech::Astronomy;
        assert_eq!(Tech::Astronomy.cost(), 50.0); // Expensive late-game tech
    }

    #[test]
    fn test_observatory_requires_astronomy() {
        assert_eq!(BuildingType::Observatory.required_tech(), Some(Tech::Astronomy));
    }

    #[test]
    fn test_observe_action_generates_knowledge() {
        let mut world = World::new();

        // Setup Resources
        let mut res = ColonyResources::default();
        res.knowledge = 0.0;
        res.max_knowledge = 100.0;
        world.insert_resource(res);

        // Setup Observatory
        let observatory = world.spawn(Observatory).id();

        // Setup Pop working there
        world.spawn((
            Pop,
            AssignedTo {
                entity: observatory,
                assignment_type: AssignmentType::ObservatoryWorker,
            },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);
        schedule.run(&mut world);

        // Check Knowledge Gain
        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge > 0.0);
    }

    #[test]
    fn test_observe_action_applies_mood_modifier() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let observatory = world.spawn(Observatory).id();
        let pop = world.spawn((
            Pop,
            Morale::default(),
            AssignedTo {
                entity: observatory,
                assignment_type: AssignmentType::ObservatoryWorker,
            },
        )).id();

        // Run system multiple times to ensure probability triggers
        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);

        // Mock RNG or run enough times to trigger effect
        // For test determinism, we might need a mocked RNG resource,
        // but for now we check if *either* modifier appears after many ticks.
        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let morale = world.get::<Morale>(pop).unwrap();
        let has_inspired = morale.modifiers.iter().any(|m| m.label == "Cosmic Inspiration");
        let has_dread = morale.modifiers.iter().any(|m| m.label == "Existential Dread");

        assert!(has_inspired || has_dread, "Should have triggered a mood modifier");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Update `Tech` Enum (`src/layer1/tech.rs`)

```rust
pub enum Tech {
    // ... existing
    Astronomy,
}

impl Tech {
    pub fn cost(&self) -> f32 {
        match self {
            // ...
            Self::Astronomy => 50.0,
        }
    }
    // Update label()
}
```

### 2. Update `BuildingType` (`src/layer1/building.rs`)

```rust
pub enum BuildingType {
    // ...
    Observatory,
}

impl BuildingType {
    pub fn required_tech(&self) -> Option<Tech> {
        match self {
            // ...
            Self::Observatory => Some(Tech::Astronomy),
            _ => None, // or existing
        }
    }
    // Update char() -> 'O'
    // Update cost() -> High Stone/Glass cost
}
```

### 3. Update `AssignmentType` (`src/layer1/actions/mod.rs`)

```rust
pub enum AssignmentType {
    // ...
    ObservatoryWorker,
}
```

### 4. Implement `Observatory` Component and Logic (`src/layer1/observatory.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::morale::{Morale, MoraleModifier};
use rand::Rng;

#[derive(Component, Default)]
pub struct Observatory;

pub fn process_observe_system(
    mut pops: Query<(Entity, &AssignedTo, &mut Morale)>,
    observatories: Query<&Observatory>,
    mut resources: ResMut<ColonyResources>,
) {
    let mut rng = rand::thread_rng();

    for (entity, assignment, mut morale) in pops.iter_mut() {
        if assignment.assignment_type == AssignmentType::ObservatoryWorker {
            if observatories.get(assignment.entity).is_ok() {
                // 1. Generate Knowledge
                resources.knowledge += 0.02;
                resources.knowledge = resources.knowledge.clamp(0.0, resources.max_knowledge);

                // 2. Chance for Overview Effect (e.g., 1% per tick)
                if rng.gen_bool(0.01) {
                    // 50/50 split for now
                    if rng.gen_bool(0.5) {
                        morale.add_modifier(MoraleModifier {
                            label: "Cosmic Inspiration".to_string(),
                            value: 0.15,
                            duration: 500,
                            source: "Observatory".to_string(),
                        });
                    } else {
                        morale.add_modifier(MoraleModifier {
                            label: "Existential Dread".to_string(),
                            value: -0.10,
                            duration: 500,
                            source: "Observatory".to_string(),
                        });
                    }
                }
            }
        }
    }
}
```

### 5. Register System

Add `process_observe_system` to the main simulation loop in `lib.rs` or `main.rs`.

## 5. REFACTOR Phase: Quality & Design

- **Trait Integration:** Traits should influence the outcome. `Optimist` or `Curious` pops should get "Inspiration" more often. `Anxious` or `Traditionalist` pops might get "Dread".
- **Layer 2 Hook:** In the future, the amount of "Dread" could scale with the number of *hostile* fleets visible in Layer 2.
- **Visuals:** Add a particle effect or log message when the effect triggers ("Pop X stared into the void...").
- **UI:** Ensure the Observatory has a unique glyph and description.

## 6. Acceptance Criteria (Testable!)

- [ ] `Tech::Astronomy` exists and costs 50 Knowledge.
- [ ] `Observatory` building requires `Astronomy`.
- [ ] Pops assigned to `Observatory` generate Knowledge.
- [ ] Pops assigned to `Observatory` occasionally receive "Cosmic Inspiration" OR "Existential Dread" mood modifiers.
- [ ] `cargo test` passes with new tests included.
- [ ] `cargo clippy` passes.

## 7. Technical Guidance

- **Randomness:** Use `rand::thread_rng()` for the mood chance. Ensure `rand` is available in `Cargo.toml` (it should be, as it is used elsewhere).
- **Clamp:** Always clamp resources to max to avoid overflow/logic errors.
- **System Order:** Run `process_observe_system` alongside `process_research_system`.

## 8. Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
