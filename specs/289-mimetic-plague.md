# Specification: The Mimetic Plague (Layer 1)

## 1. Overview
The Mimetic Plague introduces a "Memetic Hazard" that spreads through social interactions and line of sight rather than physical contact. Infected Pops become obsessed with a specific, otherwise useless task (e.g., stacking chairs, digging holes) and actively try to convince others to join them. This creates a dangerous contagion that paralyzes colony productivity without physically harming the Pops, presenting the player with a unique crisis management scenario.

## 2. Dependencies
- `003` Population Basics (Pop needs and behavior).
- `016` Utility AI System (for task assignment and prioritization).
- `047` Pop Relationships (for social transmission).

## 3. RED Phase: Tests First

```rust
// tests/mimetic_plague_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::pop::Pop;
    use scale::layer1::social::SocialInteractionEvent;
    use scale::layer1::utility_ai::UtilityAIBuffer;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems and resources
        world
    }

    #[test]
    fn test_memetic_infection_spread_via_social_interaction() {
        let mut world = setup_world();

        let infected_pop = world.spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::StackChairs,
                intensity: 1.0,
            }
        )).id();

        let healthy_pop = world.spawn((Pop,)).id();

        // Simulate a social interaction
        world.send_event(SocialInteractionEvent {
            initiator: infected_pop,
            target: healthy_pop,
        });

        // Run transmission system
        world.run_system_once(process_memetic_transmission_system);

        // Assert the healthy pop is now infected
        assert!(world.entity(healthy_pop).contains::<MemeticInfection>());
    }

    #[test]
    fn test_infected_pop_prioritizes_obsession_task() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::DigHoles,
                intensity: 1.0,
            },
            UtilityAIBuffer::default(),
        )).id();

        // Run AI evaluation
        world.run_system_once(evaluate_obsession_utility_system);

        let buffer = world.get::<UtilityAIBuffer>(pop).unwrap();
        // The obsession task should have maximum utility
        let top_action = buffer.get_top_action().unwrap();
        assert_eq!(top_action.action_type, ActionType::MemeticObsession(ObsessionType::DigHoles));
    }

    #[test]
    fn test_quarantine_prevents_transmission() {
        let mut world = setup_world();

        let infected_pop = world.spawn((
            Pop,
            MemeticInfection {
                obsession_type: ObsessionType::DigHoles,
                intensity: 1.0,
            },
            Quarantined, // Marker preventing social interaction
        )).id();

        let healthy_pop = world.spawn((Pop,)).id();

        world.send_event(SocialInteractionEvent {
            initiator: infected_pop,
            target: healthy_pop,
        });

        world.run_system_once(process_memetic_transmission_system);

        // The healthy pop should remain uninfected due to quarantine
        assert!(!world.entity(healthy_pop).contains::<MemeticInfection>());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/memetic_plague.rs

use bevy_ecs::prelude::*;
use crate::layer1::social::SocialInteractionEvent;
use crate::layer1::utility_ai::{ActionType, UtilityAIBuffer};

#[derive(Component, Debug, Clone, PartialEq)]
pub struct MemeticInfection {
    pub obsession_type: ObsessionType,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObsessionType {
    StackChairs,
    DigHoles,
    // Add more types as needed
}

#[derive(Component)]
pub struct Quarantined;

pub fn process_memetic_transmission_system(
    mut events: EventReader<SocialInteractionEvent>,
    mut commands: Commands,
    infected_query: Query<&MemeticInfection, Without<Quarantined>>,
    healthy_query: Query<Entity, (With<crate::layer1::pop::Pop>, Without<MemeticInfection>, Without<Quarantined>)>,
) {
    for event in events.read() {
        if let Ok(infection) = infected_query.get(event.initiator) {
            if healthy_query.get(event.target).is_ok() {
                // Infect the target
                commands.entity(event.target).insert(infection.clone());
            }
        }
    }
}

pub fn evaluate_obsession_utility_system(
    mut query: Query<(&MemeticInfection, &mut UtilityAIBuffer)>,
) {
    for (infection, mut buffer) in query.iter_mut() {
        // Obsession completely overrides other needs with high utility
        buffer.add_action(ActionType::MemeticObsession(infection.obsession_type.clone()), 10.0);
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Transmission Vectors:** Extend `process_memetic_transmission_system` to also trigger on line of sight or proximity, not just explicit social interaction events.
- **Cure Mechanism:** Introduce a way to cure the infection, perhaps through a Medical/Psychiatric job or specific items.
- **Integration:** Ensure `ActionType::MemeticObsession` is handled by the execution systems so Pops actually perform the visual task (e.g., moving to random tiles and playing an animation).
- **UI:** Add an indicator above infected Pops or in the Inspector UI to clearly show their mental state.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/memetic_plague.rs`.
- [ ] Memetic infections spread properly during social interactions unless Quarantined.
- [ ] Infected Pops prioritize their obsession task over normal jobs and needs.

## 7. Technical Guidance
- **Utility AI:** The obsession utility score should be high enough to override basic needs but perhaps slowly decay over time or be mitigated by strong positive morale.
- **Action Execution:** You will need to add `MemeticObsession(ObsessionType)` to the global `ActionType` enum and create a basic arrival/execution system for it so the Pops aren't just standing idle.
- **Event Reading:** Be careful with `EventReader` inside systems; ensure it runs at the right point in the schedule after social events are generated.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Builder questions:*
1. The `SocialInteractionEvent` referenced in RED/GREEN phase does not exist in the codebase. Should this be a new event, or use an existing one? If new, what triggers it?
   - *Architect:* Create a new `SocialInteractionEvent { pub initiator: Entity, pub target: Entity }` in `src/layer1/social/mod.rs` (or directly in the new `memetic_plague` module) and emit it from a new `proximity_social_system` when two pops are close.
2. `UtilityAIBuffer` does not have an `add_action` or `get_top_action` method. Utility scoring is now done via `PopDecider` returning `(ActionType, f32, Option<Entity>)`. How should we inject the obsession into this new Utility AI evaluation pipeline?
   - *Architect:* Use the new `PopDecider` pattern (e.g. `PopDecider::consider(action, utility, target)`) to return a high utility score for the obsession.
3. Adding `MemeticObsession(ObsessionType)` to `ActionType` is currently impossible because `ActionType` is a C-like enum where variants have no data (`ActionType` uses `pub const COUNT: usize = 39` and `as_index()` for array bounds). We can't easily add a variant with data like `(ObsessionType)`. How should the obsession action be represented? Perhaps multiple action types (`StackChairs`, `DigHoles`), or a single `MemeticObsession` action type that looks up the `ObsessionType` from the `MemeticInfection` component during execution?
   - *Architect:* Add a single unit variant `MemeticObsession` to `ActionType` and increase `ActionType::COUNT`. During execution, the system handling `ActionType::MemeticObsession` should query the `MemeticInfection` component on the entity to determine the specific visual task or target.