# The Golden Age (Spec 482)

## Overview
The danger of peace. Hard times create strong men; good times create soft men. Long periods of high safety and fulfilled needs generate "Complacency". Complacent pops have high mood but reduced movement speed, slower skill gain, and ignore "Low Priority" alerts.

## Dependencies
- `016` Utility AI System (Implemented)
- `031` Pop Morale (Implemented)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;

    #[test]
    fn test_complacency_accumulates_at_high_morale() {
        let mut world = World::new();

        let pop = world.spawn((
            PopBundle::default(),
            Morale { value: 0.95, ..Default::default() },
            ComplacencyState { duration: 0 },
        )).id();

        // Run system
        world.run_system_once(update_complacency_system).unwrap();

        // Assert
        let state = world.get::<ComplacencyState>(pop).unwrap();
        assert!(state.duration > 0, "Complacency duration should increase when morale is high");
    }

    #[test]
    fn test_complacent_trait_applied() {
        let mut world = World::new();

        let pop = world.spawn((
            PopBundle::default(),
            Morale { value: 0.95, ..Default::default() },
            ComplacencyState { duration: COMPLACENCY_THRESHOLD },
        )).id();

        // Run system
        world.run_system_once(update_complacency_system).unwrap();

        // Assert
        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.0.contains(&Trait::Complacent), "Pop should gain Complacent trait");
    }

    #[test]
    fn test_complacent_pop_moves_slower() {
        let traits = Traits(vec![Trait::Complacent].into_iter().collect());
        let modifier = get_trait_move_speed_modifier(&traits);
        assert!(modifier < 1.0, "Complacent pops should move slower");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
// Inside src/layer1/morale.rs or a new layer1/golden_age.rs file

pub const COMPLACENCY_THRESHOLD: u32 = 1000; // Ticks of high morale

#[derive(Component, Default, Debug)]
pub struct ComplacencyState {
    pub duration: u32,
}

// In src/layer1/traits.rs
// pub enum Trait {
// ...
// /// Ignorant of danger, moving slower and gaining skills slower.
// Complacent,
// ...
// }

// Update existing get_trait_move_speed_modifier in traits.rs to handle Trait::Complacent
// Update skill gain logic in skills.rs to handle Trait::Complacent

pub fn update_complacency_system(
    mut query: Query<(&Morale, &mut ComplacencyState, &mut Traits)>,
) {
    for (morale, mut state, mut traits) in query.iter_mut() {
        if morale.value >= 0.9 {
            state.duration += 1;
            if state.duration > COMPLACENCY_THRESHOLD && !traits.0.contains(&Trait::Complacent) {
                traits.0.insert(Trait::Complacent);
            }
        } else {
            state.duration = state.duration.saturating_sub(1);
            if state.duration == 0 && traits.0.contains(&Trait::Complacent) {
                traits.0.remove(&Trait::Complacent);
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- **Action Evaluation**: In `Utility AI`, pops with `Trait::Complacent` should deprioritize "Low Priority" tasks, such as responding to minor alerts or performing non-critical maintenance.
- **Skill Gain**: Integrate the skill gain penalty into the existing skill learning systems (e.g., `src/layer1/skills.rs`).

## Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops with sustained high morale (e.g. >= 0.9) eventually gain `Trait::Complacent`.
- [ ] `Trait::Complacent` applies a movement speed penalty.
- [ ] Loss of high morale slowly removes the `Trait::Complacent` trait.

## Technical Guidance
- Implement `ComplacencyState` and `update_complacency_system` to track sustained high morale.
- Register `update_complacency_system` in the `Layer1SystemSet::Observation` or similar update schedule.
- Update `Trait` enum in `src/layer1/traits.rs` to include `Complacent`.
- Modify `get_trait_move_speed_modifier` to return `< 1.0` for `Trait::Complacent`.
- When ignoring low priority alerts, consider checking `Trait::Complacent` in the Utility AI evaluation loop.

## Questions
*Builder: add questions here if spec is unclear. Architect will address.*
