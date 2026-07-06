# 1320: The Architect's Ego

## Overview

Master builders who refuse to follow the player's blueprints if they think they know better. Pops assigned to the "Architect" role develop an "Ego" stat. High Ego architects build structures faster and stronger, but they will sometimes ignore the player's placement or design choices, substituting their own aesthetic or structural ideas.

## Dependencies

- `009` — Job System (for `Architect` role).
- `006` — Building Placement.
- `010` — Chronicle System (logging when an Architect overrides).

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::architect_ego::{Ego, override_building_choice};
    use crate::layer1::building::{BuildingType, BuildingTask};

    #[test]
    fn test_ego_initialization() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Ego::default())).id();
        let ego = world.get::<Ego>(entity).unwrap();
        assert_eq!(ego.level, 0.0);
    }

    #[test]
    fn test_architect_override_high_ego() {
        let mut world = World::new();
        let builder = world.spawn((Pop, Ego { level: 90.0 })).id();
        let mut task = BuildingTask {
            assigned_builder: Some(builder),
            target_building: BuildingType::Bunker,
            progress: 0.0,
        };

        // Run the system/function that evaluates override
        let overriden = override_building_choice(&world, &mut task);

        // At high ego, there is a chance to override. For this test, assume a mocked rng or deterministic check where 90+ ego always overrides
        assert!(overriden, "High ego architect should override building choice");
        assert_ne!(task.target_building, BuildingType::Bunker, "Target building should be changed by ego");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/architect_ego.rs
use bevy_ecs::prelude::*;
use crate::layer1::building::{BuildingType, BuildingTask};

#[derive(Component, Default, Debug)]
pub struct Ego {
    pub level: f32, // 0.0 to 100.0
}

pub fn override_building_choice(world: &World, task: &mut BuildingTask) -> bool {
    if let Some(builder) = task.assigned_builder {
        if let Some(ego) = world.get::<Ego>(builder) {
            if ego.level > 80.0 {
                // Mock random override for now
                task.target_building = BuildingType::Plaza; // Override to Plaza
                return true;
            }
        }
    }
    false
}
```

## REFACTOR Phase: Quality & Design

- **RNG**: Implement proper seeded RNG instead of deterministic overrides.
- **Stat boost**: Implement the build speed multiplier for high ego (e.g. ego > 80 -> 2x build speed).
- **Chronicle**: Send an event when an override occurs to log it in the Chronicle.

## Acceptance Criteria

- [ ] `Ego` component exists.
- [ ] High Ego has a chance to change `BuildingTask` target.
- [ ] Tests in RED phase pass.
- [ ] `cargo check` passes.

## Technical Guidance

- Integrate `override_building_choice` in the system where building tasks begin executing.
- Use `BuildingType` variants appropriately (e.g., `Plaza` or `Monument`).

## Questions
*Builder: add questions here if spec is unclear.*
