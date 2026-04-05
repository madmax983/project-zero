# 800: Generational Dissonance

## Overview

The young no longer understand the struggles of the old. The colony fractures along age lines. Pops born on the colony (Generation 2+) do not inherit the trauma/memories of the founding generation. The founders view the youth as ungrateful, suffering morale penalties when working alongside them. The youth view the founders as paranoid, ignoring safety warnings or ancient edicts. This creates a tension between adhering to rigid, safe traditions or adapting to a relaxed, resource-efficient lifestyle.

## Dependencies

- `008` — Pop generation and traits (must exist to track generation number)
- `112` — Morale and social interactions (must exist for the penalty to apply)
- `340` — Memory/Trauma system (must exist to check inherited trauma)

## RED Phase: Tests First

```rust
// tests/layer1/generational_dissonance.rs

use scale::layer1::pops::{Pop, Generation, Morale, WorkingGroup};
use bevy::prelude::*;

#[test]
fn test_mixed_generation_workgroup_reduces_founder_morale() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, scale::layer1::social::process_generational_friction);

    // Spawn a founder pop
    let founder = app.world.spawn((
        Pop,
        Generation(1),
        Morale { current: 50, max: 100 },
        WorkingGroup { id: 1 },
    )).id();

    // Spawn a native-born pop in the same work group
    let youth = app.world.spawn((
        Pop,
        Generation(2),
        Morale { current: 50, max: 100 },
        WorkingGroup { id: 1 },
    )).id();

    // Act
    app.update();

    // Assert
    let founder_morale = app.world.get::<Morale>(founder).unwrap();
    assert!(founder_morale.current < 50, "Founder morale should decrease when working with the youth");

    let youth_morale = app.world.get::<Morale>(youth).unwrap();
    // Youth might not suffer the same penalty, or might suffer a different one
    assert_eq!(youth_morale.current, 50, "Youth morale is not directly penalized in the same way by default");
}

#[test]
fn test_youth_ignores_safety_edicts() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, scale::layer1::utility_ai::evaluate_safety_edicts);

    let youth = app.world.spawn((
        Pop,
        Generation(2),
        scale::layer1::utility_ai::EdictCompliance { ignores_safety: false },
    )).id();

    // Act
    app.update();

    // Assert
    let compliance = app.world.get::<scale::layer1::utility_ai::EdictCompliance>(youth).unwrap();
    assert!(compliance.ignores_safety, "Generation 2+ pops should passively ignore safety edicts due to lack of trauma");
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/social.rs

use bevy::prelude::*;
use crate::layer1::pops::{Generation, Morale, WorkingGroup};

pub fn process_generational_friction(
    mut query: Query<(Entity, &Generation, &WorkingGroup, &mut Morale)>,
) {
    // Simple O(N^2) for GREEN phase, finding mismatched generations in the same group
    let mut penalties: Vec<(Entity, u32)> = Vec::new();

    for (entity_a, gen_a, group_a, _) in query.iter() {
        if gen_a.0 == 1 {
            // Check if there is any pop in the same group with Generation > 1
            let mut has_youth = false;
            for (entity_b, gen_b, group_b, _) in query.iter() {
                if entity_a != entity_b && group_a.id == group_b.id && gen_b.0 > 1 {
                    has_youth = true;
                    break;
                }
            }
            if has_youth {
                penalties.push((entity_a, 5)); // Flat 5 morale penalty
            }
        }
    }

    // Apply penalties
    for (entity, penalty) in penalties {
        if let Ok((_, _, _, mut morale)) = query.get_mut(entity) {
            morale.current = morale.current.saturating_sub(penalty);
        }
    }
}
```

```rust
// src/layer1/utility_ai/edicts.rs

use bevy::prelude::*;
use crate::layer1::pops::Generation;

#[derive(Component, Default)]
pub struct EdictCompliance {
    pub ignores_safety: bool,
}

pub fn evaluate_safety_edicts(
    mut query: Query<(&Generation, &mut EdictCompliance)>,
) {
    for (gen, mut compliance) in query.iter_mut() {
        if gen.0 > 1 {
            compliance.ignores_safety = true;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: The O(N^2) check in `process_generational_friction` is too slow for large colonies. We should use a `HashMap` grouped by `WorkingGroup::id` to check the generation composition of each group in O(N) time.
- **Nuance**: The friction shouldn't just be `Generation == 1` vs `Generation > 1`. It should be based on the difference in the `Generation` value, or specific inherited trauma tags.
- **AI Integration**: The `ignores_safety` flag needs to actually modify the Utility AI scoring for actions (e.g., scoring a dangerous path higher if it's faster, ignoring the safety penalty).

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Founders suffer a morale penalty when assigned to the same working group as subsequent generations.
- [ ] Generation 2+ pops evaluate safety-related actions differently in the Utility AI.

## Technical Guidance

- Modify the `Pop` spawning logic to ensure `Generation` increments for births on the colony, while starter pops are `Generation(1)`.
- Ensure `EdictCompliance` is integrated into the `evaluate_actions_system` to actually alter behavior.

## Questions
*Builder: add questions here if spec is unclear.*
