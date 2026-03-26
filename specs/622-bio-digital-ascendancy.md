# Spec 622: The Bio-Digital Ascendancy

## 1. Overview
The slow, unsettling evolution of colonists from biological beings into a unified synthetic network. Pops with high technical skill and long lifespans can undergo "Cybernetic Integration." Integrated Pops (souls) lose their need for food or rest, but their individuality decreases, merging their `UtilityWeights` into a colony-wide average.

## 2. Dependencies
- `src/layer1/pop.rs` (Pop components)
- `src/layer1/needs.rs` (Hunger, Rest)
- `src/layer1/utility_ai.rs` (UtilityWeights)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{Hunger, Rest};
    use crate::layer1::utility_ai::UtilityWeights;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_cybernetic_integration_removes_biological_needs() {
        let mut app = App::new();
        app.add_systems(Update, cybernetic_integration_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Hunger { value: 50.0 },
            Rest { value: 50.0 },
            CyberneticIntegration::default(),
        )).id();

        app.update();

        assert!(app.world().get::<Hunger>(pop_entity).is_none(), "Integrated Pop should not have Hunger");
        assert!(app.world().get::<Rest>(pop_entity).is_none(), "Integrated Pop should not have Rest");
    }

    #[test]
    fn test_utility_weights_merge_towards_average() {
        let mut app = App::new();
        app.insert_resource(ColonyAverageUtility { weights: UtilityWeights { work: 0.8, leisure: 0.2, social: 0.5 } });
        app.add_systems(Update, cybernetic_mind_merge_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            UtilityWeights { work: 0.2, leisure: 0.9, social: 0.1 },
            CyberneticIntegration { integration_level: 0.5 },
        )).id();

        app.update();

        let weights = app.world().get::<UtilityWeights>(pop_entity).unwrap();
        // Should move towards the average
        assert!(weights.work > 0.2);
        assert!(weights.leisure < 0.9);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::{Hunger, Rest};
use crate::layer1::utility_ai::UtilityWeights;

#[derive(Component, Default)]
pub struct CyberneticIntegration {
    pub integration_level: f32, // 0.0 to 1.0
}

#[derive(Resource)]
pub struct ColonyAverageUtility {
    pub weights: UtilityWeights,
}

pub fn cybernetic_integration_system(
    mut commands: Commands,
    query: Query<Entity, (With<CyberneticIntegration>, Or<(With<Hunger>, With<Rest>)>)>
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<Hunger>().remove::<Rest>();
    }
}

pub fn cybernetic_mind_merge_system(
    avg: Res<ColonyAverageUtility>,
    mut query: Query<(&mut UtilityWeights, &CyberneticIntegration)>
) {
    for (mut weights, integration) in query.iter_mut() {
        let blend = integration.integration_level;
        weights.work = weights.work * (1.0 - blend) + avg.weights.work * blend;
        weights.leisure = weights.leisure * (1.0 - blend) + avg.weights.leisure * blend;
        weights.social = weights.social * (1.0 - blend) + avg.weights.social * blend;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate `ColonyAverageUtility` dynamically based on all integrated pops rather than relying on a static resource.
- Add events for when a Pop becomes fully integrated (for UI and chronicle).
- Consider how cybernetics affects the `Health` component and `Death` conditions.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Cybernetic pops do not decay hunger or rest.

## 7. Technical Guidance
- Register `CyberneticIntegration` component and systems in the main simulation schedule.
- Ensure the mind merge system runs after utility weights might normally be modified by memories.

## 8. Questions
*Builder: add questions here if spec is unclear.*
