# 097: Social Need and Tavern

## Overview

Pops now have a `Leisure` need. When this need is low, they will seek out a `Tavern` to socialize. This adds a new layer of psychological needs beyond basic survival (Hunger/Rest) and introduces the first social building.

## Dependencies

- `005` — Pop needs (need structure)
- `006` — Building placement
- `016` — Utility AI System (action selection)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/social_tests.rs - Create new test file

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::needs::{Needs, decay_needs_system};
    use crate::layer1::pop::{Pop, GridPosition};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::utility_ai::{ActionType, UtilityWeights, calculate_context_score};
    use crate::layer1::social::{Tavern, evaluate_socialize, restore_leisure_system}; // New module

    #[test]
    fn test_needs_has_leisure() {
        let needs = Needs::default();
        // Leisure starts high like others
        assert_eq!(needs.leisure, 0.8);
    }

    #[test]
    fn test_leisure_decays() {
        let mut world = World::new();
        world.spawn((Pop, Needs::default()));

        decay_needs_system(&mut world);

        let needs = world.query::<&Needs>().single(&world);
        assert!(needs.leisure < 0.8, "Leisure should decay");
    }

    #[test]
    fn test_tavern_component_defaults() {
        let tavern = Tavern::default();
        assert_eq!(tavern.capacity, 5); // Taverns hold more people than houses
        assert!(tavern.visitors.is_empty());
    }

    #[test]
    fn test_evaluate_socialize_finds_tavern() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs { leisure: 0.2, ..Default::default() }; // Low leisure
        let weights = UtilityWeights::default();

        let tavern_entity = world.spawn((
            Building { building_type: BuildingType::Tavern },
            GridPosition { x: 5, y: 0 },
            Tavern::default(),
        )).id();

        let taverns = world.query::<(Entity, &GridPosition, &Tavern)>();

        let result = evaluate_socialize(&pop_pos, &needs, &weights, &taverns);

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, tavern_entity);
        assert!(utility > 0.5, "Utility should be high for low leisure");
    }

    #[test]
    fn test_evaluate_socialize_ignored_when_leisure_high() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs { leisure: 0.9, ..Default::default() };
        let weights = UtilityWeights::default();

        world.spawn((
            Building { building_type: BuildingType::Tavern },
            GridPosition { x: 5, y: 0 },
            Tavern::default(),
        ));

        let taverns = world.query::<(Entity, &GridPosition, &Tavern)>();

        // Should produce very low utility or None depending on curve
        let result = evaluate_socialize(&pop_pos, &needs, &weights, &taverns);

        if let Some((utility, _)) = result {
            assert!(utility < 0.2, "High leisure should result in low utility");
        }
    }

    #[test]
    fn test_restore_leisure_system() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            Needs { leisure: 0.2, ..Default::default() },
        )).id();

        let mut tavern = Tavern::default();
        tavern.visitors.push(pop);
        world.spawn(tavern);

        restore_leisure_system(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure > 0.2, "Leisure should be restored");
        assert!(needs.leisure <= 1.0);
    }

    #[test]
    fn test_action_type_socialize_variant() {
        let social = ActionType::Socialize;
        assert_eq!(social, ActionType::Socialize);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Needs Struct
```rust
// src/layer1/needs.rs

#[derive(Component, Clone, Copy, Debug)]
pub struct Needs {
    pub hunger: f32,
    pub rest: f32,
    pub leisure: f32, // New field
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            hunger: 0.8,
            rest: 0.8,
            leisure: 0.8,
        }
    }
}

// Update decay_needs_system
pub fn decay_needs_system(world: &mut World) {
    let mut query = world.query::<&mut Needs>();
    for mut needs in query.iter_mut(world) {
        needs.hunger = (needs.hunger - HUNGER_DECAY).max(0.0);
        needs.rest = (needs.rest - REST_DECAY).max(0.0);
        needs.leisure = (needs.leisure - 0.015).max(0.0); // Slightly slower decay than hunger
    }
}
```

### 2. Add Tavern Component and Module
```rust
// src/layer1/social.rs (New file)

use bevy_ecs::prelude::*;
use super::pop::GridPosition;
use super::needs::Needs;
use super::utility_ai::{UtilityWeights, calculate_context_score, need_response_curve, calculate_success_modifier, ActionType};

#[derive(Component)]
pub struct Tavern {
    pub capacity: usize,
    pub visitors: Vec<Entity>,
}

impl Default for Tavern {
    fn default() -> Self {
        Self {
            capacity: 5,
            visitors: Vec::new(),
        }
    }
}

pub fn evaluate_socialize(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    taverns: &Query<(Entity, &GridPosition, &Tavern)>,
) -> Option<(f32, Entity)> {
    let urgency = need_response_curve(needs.leisure);
    let mut best: Option<(f32, Entity)> = None;

    for (entity, pos, tavern) in taverns.iter() {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            tavern.capacity,
            tavern.visitors.len(),
            weights
        );

        let success = calculate_success_modifier(ActionType::Socialize, weights);
        let utility = urgency * context * success;

        if best.is_none() || utility > best.unwrap().0 {
            best = Some((utility, entity));
        }
    }
    best
}

pub fn restore_leisure_system(world: &mut World) {
    let mut taverns = world.query::<&mut Tavern>();
    let mut visitors_to_update = Vec::new();

    // Collect visitors to avoid borrowing conflicts
    for tavern in taverns.iter(world) {
        for &visitor in &tavern.visitors {
            visitors_to_update.push(visitor);
        }
    }

    // Restore leisure
    for visitor in visitors_to_update {
        if let Some(mut needs) = world.get_mut::<Needs>(visitor) {
            needs.leisure = (needs.leisure + 0.05).min(1.0);
        }
    }
}
```

### 3. Update BuildingType
```rust
// src/layer1/building.rs
pub enum BuildingType {
    Housing,
    Farm,
    Stockpile,
    LumberMill,
    StoneMason,
    Tavern, // New
}

// Update try_place_building to attach Tavern component
match building_type {
    // ...
    BuildingType::Tavern => {
        entity.insert(Tavern::default());
    }
}
```

### 4. Integrate into Main Loop and Utility AI
- Add `mod social;` to `layer1/mod.rs`.
- Update `evaluate_actions_system` in `utility_ai.rs` to call `evaluate_socialize`.
- Add `restore_leisure_system` to `main.rs` loop.

## REFACTOR Phase: Quality & Design

- **Visitor Tracking**: Currently `Tavern.visitors` is manually populated in tests. `HTN` operator (future spec) will handle adding/removing pops from `visitors` list when they arrive/leave.
- **Social Bonus**: Future update could boost leisure restoration if more people are in the tavern (the "Social" part).
- **Drinks**: Future update could consume "Ale" resource for faster restoration or mood buffs.

## Acceptance Criteria

- [ ] `Needs` has `leisure` field that decays.
- [ ] `Tavern` building can be placed.
- [ ] `ActionType::Socialize` is selected by Utility AI when leisure is low.
- [ ] `restore_leisure_system` increases leisure for visitors.
- [ ] Tests pass with ≥85% coverage.

## Technical Guidance

- **ActionType Enum**: Ensure `ActionType` derives `Hash`, `Eq`, `PartialEq`, `Clone`, `Copy` to work with `UtilityWeights`.
- **System Order**: `restore_leisure_system` should run before `decay_needs_system` so pops get full benefit of the tick.
