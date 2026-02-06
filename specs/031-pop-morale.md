# 031: Pop Morale

## Overview

Aggregate individual needs (Hunger, Rest, Leisure) into a single `Morale` score. High morale boosts work efficiency, while low morale reduces it. This connects the "Sim" layer (needs) to the "Colony" layer (production).

## Dependencies

- `005` — Needs (Hunger, Rest)
- `028` — Social Tavern (Leisure)
- `030` — Tool Economy (for efficiency calculation hook)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/morale_tests.rs (or in needs.rs)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::needs::{Needs, get_morale_efficiency};

    #[test]
    fn test_calculate_morale() {
        let needs = Needs { hunger: 1.0, rest: 1.0, leisure: 1.0 };
        assert!((needs.morale() - 1.0).abs() < f32::EPSILON);

        let needs_mixed = Needs { hunger: 0.5, rest: 0.5, leisure: 0.5 };
        assert!((needs_mixed.morale() - 0.5).abs() < f32::EPSILON);

        let needs_bad = Needs { hunger: 0.0, rest: 0.0, leisure: 0.0 };
        assert!((needs_bad.morale() - 0.0).abs() < f32::EPSILON);

        // Uneven
        let needs_uneven = Needs { hunger: 1.0, rest: 0.0, leisure: 0.5 };
        assert!((needs_uneven.morale() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_morale_efficiency_bonus() {
        // High morale (>= 0.8) -> 1.2x speed
        assert_eq!(get_morale_efficiency(0.9), 1.2);
        assert_eq!(get_morale_efficiency(0.8), 1.2);
    }

    #[test]
    fn test_morale_efficiency_neutral() {
        // Normal morale (0.2 < m < 0.8) -> 1.0x speed
        assert_eq!(get_morale_efficiency(0.5), 1.0);
        assert_eq!(get_morale_efficiency(0.79), 1.0);
        assert_eq!(get_morale_efficiency(0.21), 1.0);
    }

    #[test]
    fn test_morale_efficiency_penalty() {
        // Low morale (<= 0.2) -> 0.5x speed
        assert_eq!(get_morale_efficiency(0.1), 0.5);
        assert_eq!(get_morale_efficiency(0.2), 0.5);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Morale Helper

```rust
// src/layer1/needs.rs

impl Needs {
    /// Calculates aggregate morale score (0.0 to 1.0).
    pub fn morale(&self) -> f32 {
        (self.hunger + self.rest + self.leisure) / 3.0
    }
}

/// Returns work efficiency multiplier based on morale.
pub fn get_morale_efficiency(morale: f32) -> f32 {
    if morale >= 0.8 {
        1.2
    } else if morale <= 0.2 {
        0.5
    } else {
        1.0
    }
}
```

### 2. Update Work Execution

Modify `src/layer1/execution.rs`:

```rust
// In work_execution_system:

use crate::layer1::needs::{Needs, get_morale_efficiency};

pub fn work_execution_system(world: &mut World) {
    // ... setup (tools, etc) ...

    // Collect morale data to avoid borrow conflicts
    let pop_morales: HashMap<Entity, f32> = world.query::<(Entity, &Needs)>()
        .iter(world)
        .map(|(e, n)| (e, n.morale()))
        .collect();

    // ... workers loop ...

    for (pop_entity, designation_entity) in workers {
        // Get base work amount (e.g. from Tool logic)
        // let tool_efficiency = ...;
        // let mut work_amount = WORK_PER_TICK * tool_efficiency;

        // Apply Morale Modifier
        if let Some(morale) = pop_morales.get(&pop_entity) {
            work_amount *= get_morale_efficiency(*morale);
        }

        // ... call mine_rock / chop_tree ...
    }
}
```

## REFACTOR Phase: Quality & Design

- **UI**: Add a Morale indicator to `render_info_panel` (avg colony morale).
- **Constants**: Extract thresholds (`MORALE_HIGH_THRESHOLD`, `MORALE_LOW_THRESHOLD`).
- **Feedback**: Maybe change pop color slightly based on morale, not just worst need? (Visual complexity vs clarity).

## Acceptance Criteria

- [ ] `Needs::morale()` returns average of needs.
- [ ] Efficiency multipliers are correct (1.2, 1.0, 0.5).
- [ ] Work execution applies morale modifier.
- [ ] Info panel shows average colony morale.
- [ ] Tests pass.

## Technical Guidance

- Ensure `HashMap` is imported in `execution.rs`.
- When calculating average morale for UI, handle the case of 0 pops (divide by zero check).

## Questions

*Builder: add questions here if spec is unclear.*
