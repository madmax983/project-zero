# 197: Civic Ideology

## Overview

The colony is defined by its core philosophy. The player selects a "Colony Goal" or "Civic Ideology" (e.g., `Survivalist`, `Technocratic`, `Industrialist`). This choice provides passive morale bonuses when the colony acts in accordance with the ideology, and penalties when it fails to live up to it.

## Dependencies

- `031` Pop Morale
- `008` Resource Stockpiles
- `011` Tech Tree

## RED Phase: Tests First

These tests define the behavior of the `CivicIdeology` system. They must be written and fail before any implementation.

```rust
// src/layer1/civic_ideology_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::pop::Pop;
    use crate::layer1::morale::{Morale, MoodModifier};
    use crate::layer1::civic_ideology::{CivicIdeology, IdeologyType, evaluate_civic_ideology_system};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_survivalist_happiness_with_abundant_food() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology { selected: IdeologyType::Survivalist });
        world.insert_resource(ColonyResources {
            food: 100.0,
            ..ColonyResources::default()
        });

        // Spawn 10 pops. Need 10 * 5 = 50 food for bonus. We have 100.
        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(pop_morale.modifiers.iter().any(|m| m.label == "Ideological Satisfaction" && m.value > 0.0));
    }

    #[test]
    fn test_survivalist_anger_with_low_food() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology { selected: IdeologyType::Survivalist });
        world.insert_resource(ColonyResources {
            food: 10.0, // 1 per pop, below threshold of 2
            ..ColonyResources::default()
        });

        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(pop_morale.modifiers.iter().any(|m| m.label == "Ideological Disappointment" && m.value < 0.0));
    }

    #[test]
    fn test_technocratic_happiness_with_high_knowledge() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology { selected: IdeologyType::Technocratic });
        world.insert_resource(ColonyResources {
            knowledge: 60.0, // Threshold 50
            ..ColonyResources::default()
        });

        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(pop_morale.modifiers.iter().any(|m| m.label == "Ideological Satisfaction"));
    }

    #[test]
    fn test_industrialist_happiness_with_high_materials() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology { selected: IdeologyType::Industrialist });
        world.insert_resource(ColonyResources {
            metal: 20.0,
            planks: 20.0,
            blocks: 20.0, // Total 60 > 50
            ..ColonyResources::default()
        });

        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(pop_morale.modifiers.iter().any(|m| m.label == "Ideological Satisfaction"));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `CivicIdeology` Resource

```rust
// src/layer1/civic_ideology.rs

use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::pop::Pop;
use crate::layer1::morale::{Morale, MoodModifier};

#[derive(Resource, Default)]
pub struct CivicIdeology {
    pub selected: IdeologyType,
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum IdeologyType {
    #[default]
    Survivalist,
    Technocratic,
    Industrialist,
}
```

### 2. `evaluate_civic_ideology_system`

```rust
pub fn evaluate_civic_ideology_system(
    ideology: Res<CivicIdeology>,
    resources: Res<ColonyResources>,
    pops: Query<Entity, With<Pop>>,
    mut morale_query: Query<&mut Morale>,
) {
    let pop_count = pops.iter().count() as f32;
    if pop_count == 0.0 { return; }

    let (modifier_value, label) = match ideology.selected {
        IdeologyType::Survivalist => {
            let total_food = resources.total_food(); // Assuming total_food() exists or use food + rations
            if total_food >= pop_count * 5.0 {
                (0.1, "Ideological Satisfaction")
            } else if total_food < pop_count * 2.0 {
                (-0.1, "Ideological Disappointment")
            } else {
                (0.0, "")
            }
        },
        IdeologyType::Technocratic => {
            if resources.knowledge >= 50.0 {
                (0.1, "Ideological Satisfaction")
            } else if resources.knowledge < 10.0 {
                (-0.1, "Ideological Disappointment")
            } else {
                (0.0, "")
            }
        },
        IdeologyType::Industrialist => {
            let total_mats = resources.metal + resources.planks + resources.blocks;
            if total_mats >= 50.0 {
                (0.1, "Ideological Satisfaction")
            } else if total_mats < 10.0 {
                (-0.1, "Ideological Disappointment")
            } else {
                (0.0, "")
            }
        },
    };

    if modifier_value != 0.0 && !label.is_empty() {
        for mut morale in morale_query.iter_mut() {
             // For MVP, simply add a 1-tick modifier.
             // Ideally we should check if it exists and update duration, but existing morale system handles stacking?
             // Specs 031 says modifiers are a Vec. If we add every tick, we might overflow.
             // GREEN phase optimization: Remove old "Ideological" modifiers first.
             morale.modifiers.retain(|m| !m.label.starts_with("Ideological"));

             morale.add_modifier(MoodModifier {
                 label: label.to_string(),
                 value: modifier_value,
                 duration: 1,
             });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Avoid String Allocations**: `MoodModifier` uses `String` for labels. Create a `const` or `enum` for modifier types to avoid `to_string()` every tick.
- **System Scheduling**: Run this system once per "Day" or "Hour" (using `SimulationTime`) rather than every tick, and set `duration` accordingly. Or use `RunCriteria`.
- **Config**: Move thresholds (5.0 food/pop, 50 knowledge) to `CivicIdeologyConfig` resource.
- **UI**: Need a way for player to select ideology (handled in `026-main-menu` or separate UI).
- **Integration**: Add `evaluate_civic_ideology_system` to the main simulation schedule in `src/simulation.rs`.

## Acceptance Criteria

- [ ] `CivicIdeology` resource exists.
- [ ] `evaluate_civic_ideology_system` correctly evaluates Survivalist, Technocratic, and Industrialist conditions.
- [ ] Pops receive `MoodModifier` with correct positive/negative values.
- [ ] Modifiers are cleared/updated correctly to prevent infinite stacking.
- [ ] All RED phase tests pass.

## Questions

- Should switching ideology be allowed mid-game? (Assume no for MVP, or with penalty).
- Should "Satisfaction" scale with success margin? (e.g. +0.2 for *very* high food). Keep simple (+0.1) for now.
  - *Architect:* No, use a flat satisfaction bonus for the MVP.
