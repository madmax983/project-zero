# 1321: The Heirloom Debt

## Overview

Pops can accumulate personal "Debt" to the colony (e.g., for expensive medical treatments or luxury goods). When a Pop dies, their Debt is passed on to their children. Pops with massive inherited Debt start with a "Desperate" trait, working harder but turning to crime if they can't pay it off.

## Dependencies

- `004` — Pop Entity
- `010` — Chronicle System (logging the inheritance of debt)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::heirloom_debt::{Debt, Desperate, process_heirloom_debt};

    #[test]
    fn test_debt_initialization() {
        let mut world = World::new();
        let entity = world.spawn((Pop::new(), Debt::default())).id();
        let debt = world.get::<Debt>(entity).unwrap();
        assert_eq!(debt.amount, 0.0);
    }

    #[test]
    fn test_debt_inheritance() {
        let mut world = World::new();

        let child_entity = world.spawn((Pop::new(), Debt { amount: 0.0 })).id();

        // Mock a parent passing away with debt
        let parent_debt = 5000.0;

        // System should apply parent debt to child
        process_heirloom_debt(&mut world, child_entity, parent_debt);

        let child_debt = world.get::<Debt>(child_entity).unwrap();
        assert_eq!(child_debt.amount, 5000.0);
    }

    #[test]
    fn test_desperate_trait_applied_on_high_debt() {
        let mut world = World::new();

        let child_entity = world.spawn((Pop::new(), Debt { amount: 0.0 })).id();

        // Massive debt should trigger Desperate trait
        let massive_debt = 20000.0;
        process_heirloom_debt(&mut world, child_entity, massive_debt);

        assert!(world.get::<Desperate>(child_entity).is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/heirloom_debt.rs
use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;

#[derive(Component, Default, Debug)]
pub struct Debt {
    pub amount: f32,
}

#[derive(Component, Default, Debug)]
pub struct Desperate;

pub fn process_heirloom_debt(world: &mut World, heir_entity: Entity, inherited_debt: f32) {
    if let Some(mut debt) = world.get_mut::<Debt>(heir_entity) {
        debt.amount += inherited_debt;

        if debt.amount >= 10000.0 {
            world.entity_mut(heir_entity).insert(Desperate);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Family Trees**: The inheritance system needs a robust way to traverse family lineages in the ECS to automatically find the correct heirs upon a Pop's death.
- **Trait Effects**: Connect the `Desperate` component to utility AI weightings so they prioritize high-yield or illicit jobs.
- **Chronicle Integration**: Send an event to the chronicle system when a massive debt is passed down, generating history lore.

## Acceptance Criteria

- [ ] `Debt` and `Desperate` components exist.
- [ ] Tests in RED phase pass.
- [ ] High inherited debt correctly adds the `Desperate` trait to a Pop.
- [ ] `cargo test` returns 0 failures.
- [ ] Test coverage ≥85% for new code.
- [ ] `cargo clippy -- -D warnings` passes.

## Technical Guidance

- Integrate `process_heirloom_debt` with the death/cleanup systems in Layer 1.
- Utilize existing family link components if they exist, or design a minimal relationship lookup mechanism.
- Consider what happens if a Pop dies with debt but no heirs (does the debt vanish, or does the colony absorb it?).

## Questions
*Builder: add questions here if spec is unclear.*
