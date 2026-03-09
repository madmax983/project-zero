# 193 Polyglot Confusion

## 1. Overview

**Fantasy:** "We don't speak the same language. The Tower of Babel in space."

As the colony grows and diverse groups arrive (or develop), communication becomes a bottleneck. Pops speak different "Dialects". When Pops with different dialects work together on the same task or in the same building, they suffer a **Coordination Penalty** to work speed, unless they have learned each other's language.

**Why:** Adds depth to workforce management. Encourages segregating workforces by origin OR investing in education/time to integrate them.

## 2. Dependencies

- `003` Pop Entity (for `Pop` component)
- `006` Job Assignment (for `Job` component)
- `066` Building Work AI (for `work_execution_system`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::Skills;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_pop_has_dialect_component() {
        let mut world = World::new();
        // Assume `Dialect::default()` is `Dialect::Common`
        let pop = world.spawn((Pop, Dialect::default())).id();

        let dialect = world.get::<Dialect>(pop).unwrap();
        assert_eq!(*dialect, Dialect::Common);
    }

    #[test]
    fn test_pop_has_linguistics_component() {
        let mut world = World::new();
        let pop = world.spawn((Pop, Linguistics::default())).id();

        let ling = world.get::<Linguistics>(pop).unwrap();
        // Should always know their native dialect
        // Ideally, Linguistics::new(native) handles this, but default might just be empty + native from Dialect component
        assert!(ling.known_dialects.is_empty(), "Default linguistics starts empty (native handled by Dialect)");
    }

    #[test]
    fn test_coordination_penalty_calculation() {
        // Helper function to calculate penalty
        // 1.0 = No Penalty
        // 0.75 = Penalty

        // Case 1: Same Dialect -> 1.0
        let penalty1 = calculate_coordination_penalty(
            &Dialect::Common,
            &Linguistics::default(),
            &Dialect::Common
        );
        assert_eq!(penalty1, 1.0);

        // Case 2: Different Dialect, Unknown -> 0.75
        let penalty2 = calculate_coordination_penalty(
            &Dialect::Common,
            &Linguistics::default(),
            &Dialect::Spacer
        );
        assert_eq!(penalty2, 0.75);

        // Case 3: Different Dialect, Known -> 1.0
        let mut ling = Linguistics::default();
        ling.known_dialects.insert(Dialect::Spacer);
        let penalty3 = calculate_coordination_penalty(
            &Dialect::Common,
            &ling,
            &Dialect::Spacer
        );
        assert_eq!(penalty3, 1.0);
    }

    #[test]
    fn test_group_coordination_penalty() {
        // If multiple workers, we take the LOWEST score against any co-worker?
        // Or average?
        // Spec: "If ANY co-worker has an unknown dialect, apply penalty."

        let mut world = World::new();
        let p1 = world.spawn((Dialect::Common, Linguistics::default())).id();
        let p2 = world.spawn((Dialect::Spacer, Linguistics::default())).id();
        let p3 = world.spawn((Dialect::Common, Linguistics::default())).id();

        // P1 working with P2 -> Penalty
        let mod1 = get_group_coordination_modifier(&world, p1, vec![p2]);
        assert_eq!(mod1, 0.75);

        // P1 working with P3 -> No Penalty
        let mod2 = get_group_coordination_modifier(&world, p1, vec![p3]);
        assert_eq!(mod2, 1.0);

        // P1 working with P2 AND P3 -> Penalty (weakest link)
        let mod3 = get_group_coordination_modifier(&world, p1, vec![p2, p3]);
        assert_eq!(mod3, 0.75);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use std::collections::HashSet;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Dialect {
    #[default]
    Common,
    Spacer,   // High-G / Void-born
    MinerCant, // Deep crust workers
    OldTongue, // Ancient/Ritualistic
    Binary,    // Cyborgs/Drones
}

#[derive(Component, Debug, Clone, Default)]
pub struct Linguistics {
    /// Dialects known fluently enough to work without penalty.
    pub known_dialects: HashSet<Dialect>,
    /// Progress towards learning a new dialect (0.0 to 1.0).
    /// Gained via social interaction or working with speakers.
    pub learning_progress: std::collections::HashMap<Dialect, f32>,
}

/// Constant for the penalty multiplier.
pub const COORDINATION_PENALTY_MULTIPLIER: f32 = 0.75;

/// Calculates the coordination modifier for a single pair.
pub fn calculate_coordination_penalty(
    my_dialect: &Dialect,
    my_linguistics: &Linguistics,
    other_dialect: &Dialect,
) -> f32 {
    if my_dialect == other_dialect {
        return 1.0;
    }
    if my_linguistics.known_dialects.contains(other_dialect) {
        return 1.0;
    }
    COORDINATION_PENALTY_MULTIPLIER
}

/// Calculates the coordination modifier for a pop working with a group.
/// Returns the worst-case penalty (weakest link logic).
pub fn get_group_coordination_modifier(
    world: &World,
    pop_entity: Entity,
    co_workers: Vec<Entity>,
) -> f32 {
    let (my_dialect, my_ling) = if let Ok((d, l)) = world.query::<(&Dialect, &Linguistics)>().get(world, pop_entity) {
        (d, l)
    } else {
        return 1.0; // Default if components missing
    };

    for coworker in co_workers {
        if let Ok(other_dialect) = world.query::<&Dialect>().get(world, coworker) {
            let mod_val = calculate_coordination_penalty(my_dialect, my_ling, other_dialect);
            if mod_val < 1.0 {
                return mod_val; // Short-circuit on first issue
            }
        }
    }
    1.0
}
```

## 5. REFACTOR Phase: Quality & Design

-   **Work Execution Integration**: In `work_execution_system` (Spec 066), identify all pops working on the same target entity (Designation or Building). Pass this list to `get_group_coordination_modifier`.
-   **Optimization**: Instead of N^2 checks every tick, cache the "Work Group" state or only check periodically (e.g., every 10 ticks).
-   **Social Integration**: In `social_system` (Spec 047), when pops interact:
    -   If dialects differ, increase `learning_progress` for both.
    -   If progress > 1.0, add to `known_dialects` and send a notification.
-   **UI**: Show "Dialect Mismatch" icon in the Pop Inspector or World Tooltip when selecting a worker.

## 6. Acceptance Criteria

- [ ] `Dialect` and `Linguistics` components exist.
- [ ] Pops spawn with `Dialect::Common` by default.
- [ ] `calculate_coordination_penalty` returns `0.75` for unknown mismatch, `1.0` otherwise.
- [ ] `get_group_coordination_modifier` correctly identifies the penalty in a mixed group.
- [ ] Tests passed for the above logic.

## 7. Technical Guidance

-   **Data Structure**: Use `HashSet` for `known_dialects` for O(1) lookups.
-   **Integration Point**: Modify `process_single_worker` in `src/layer1/execution.rs`.
    -   Currently, it processes one worker at a time.
    -   To enable group checks, `work_execution_system` should pre-group workers by `mt.target_entity`.
    -   `let workers_by_target: HashMap<Entity, Vec<Entity>> = ...;`
    -   Then iterate and apply modifiers.
-   **Migration**: Existing save files/entities need `Dialect::default()` added.

## 8. Questions

- **Q**: Should "Common" be learnable?
- *Architect:* Yes, if a Pop starts with *only* MinerCant (e.g., a rescue pod survivor), they need to learn Common to work efficiently with the main colony.
- **Q**: Does the penalty stack?
- *Architect:* No. It's a flat "Coordination Penalty". Being confused by 1 person is the same state as being confused by 5. Simplicity first.
