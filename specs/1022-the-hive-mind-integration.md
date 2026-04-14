# 1022: The Hive Mind Integration

## 1. Overview
Researching "Xeno-Integration" allows implanting Pops with local flora, turning them into a "Collective." Integrated Pops stop needing Sleep or Leisure and become immune to planetary hazards, but lose their individual personality traits. However, tension arises because the integrated Collective evaluates non-integrated Pops as "inefficient" and may hostilely interact with them (e.g., refusing to feed them).

## 2. Dependencies
- Layer 1 `Research` system.
- Layer 1 `Pop` entity (`Traits`, `Needs`).
- Layer 1 `Utility AI` (Evaluating targets/actions).
- Layer 1 `Medical` or `Surgery` interactions.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, TraitList};
    use crate::layer1::needs::{Sleep, Leisure, Hunger};
    use crate::layer1::medical::SurgeryEvent;
    use crate::layer1::utility_ai::{ActionType, Score};

    #[test]
    fn test_xeno_integration_removes_needs_and_traits() {
        let mut app = App::new();
        app.add_event::<SurgeryEvent>();
        app.add_systems(Update, process_integration_surgery_system);

        let pop = app.world_mut().spawn((
            Pop,
            TraitList { traits: vec!["Lazy".to_string(), "Brave".to_string()] },
            Sleep { value: 50.0 },
            Leisure { value: 50.0 },
        )).id();

        app.world_mut().resource_mut::<Events<SurgeryEvent>>().send(SurgeryEvent {
            patient: pop,
            procedure: "XenoIntegration".to_string(),
        });

        app.update();

        assert!(app.world().get::<IntegratedCollective>(pop).is_some(), "Pop should gain the IntegratedCollective component.");
        assert!(app.world().get::<Sleep>(pop).is_none(), "Integrated pops should not need sleep.");
        assert!(app.world().get::<Leisure>(pop).is_none(), "Integrated pops should not need leisure.");
        let traits = app.world().get::<TraitList>(pop).unwrap();
        assert!(traits.traits.is_empty(), "Integrated pops should lose individual traits.");
    }

    #[test]
    fn test_collective_evaluates_unintegrated_pops_as_inefficient() {
        let mut app = App::new();
        app.add_systems(Update, score_feeding_action_system);

        let integrated = app.world_mut().spawn((Pop, IntegratedCollective)).id();
        let unintegrated = app.world_mut().spawn((Pop, Hunger { value: 0.0, decay_rate: 1.0 })).id();
        let another_integrated = app.world_mut().spawn((Pop, IntegratedCollective, Hunger { value: 0.0, decay_rate: 1.0 })).id();

        // Simulate Utility AI evaluating a "Feed Others" action
        // For unintegrated target
        let score_unintegrated = evaluate_feed_score(integrated, unintegrated, app.world());
        // For integrated target
        let score_integrated = evaluate_feed_score(integrated, another_integrated, app.world());

        assert!(score_unintegrated < score_integrated, "Integrated pops should heavily penalize helping unintegrated pops.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/hive_mind_integration.rs
use bevy::prelude::*;
use crate::layer1::pop::{Pop, TraitList};
use crate::layer1::needs::{Sleep, Leisure, Hunger};
use crate::layer1::medical::SurgeryEvent;

#[derive(Component)]
pub struct IntegratedCollective;

pub fn process_integration_surgery_system(
    mut commands: Commands,
    mut events: EventReader<SurgeryEvent>,
    mut query: Query<&mut TraitList, With<Pop>>,
) {
    for event in events.read() {
        if event.procedure == "XenoIntegration" {
            // Add collective flag
            commands.entity(event.patient).insert(IntegratedCollective);

            // Remove needs
            commands.entity(event.patient).remove::<Sleep>();
            commands.entity(event.patient).remove::<Leisure>();

            // Wipe personality
            if let Ok(mut traits) = query.get_mut(event.patient) {
                traits.traits.clear();
            }
        }
    }
}

// Helper for test logic; in real code this would be integrated into the Utility AI scorer
pub fn evaluate_feed_score(actor: Entity, target: Entity, world: &World) -> f32 {
    let mut base_score = 100.0;

    let actor_is_collective = world.get::<IntegratedCollective>(actor).is_some();
    let target_is_collective = world.get::<IntegratedCollective>(target).is_some();

    if actor_is_collective && !target_is_collective {
        // Massive penalty to helping the "inefficient"
        base_score -= 90.0;
    }

    base_score
}

pub fn score_feeding_action_system() {
    // Stub for the actual Utility AI system integration
}
```

## 5. REFACTOR Phase: Quality & Design
- **Hazard Immunity:** Add logic to the environment damage systems to bypass entities with `IntegratedCollective`.
- **Hive Mind Needs:** While they lose Sleep and Leisure, they might gain a new unified need, like `BiomassNodeConnection` or `SynapseProximity`, requiring new infrastructure.
- **Faction Split:** If a high enough percentage of the colony is integrated, the `Collective` should probably formally split into its own Faction and declare a silent civil war against the remaining unintegrated Pops.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_xeno_integration_removes_needs_and_traits` passes.
- [ ] Test `test_collective_evaluates_unintegrated_pops_as_inefficient` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Removing components dynamically (`commands.entity().remove()`) requires care if other systems strictly require those components to function (e.g., if the main needs decay system panics on missing `Sleep`). The systems might need to use `Option<&mut Sleep>` instead.

## 8. Questions
*Builder: add questions here if spec is unclear.*
