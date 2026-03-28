# 710: The Engram Economy

## Overview

Desperation demands sacrifice. To keep the empire solvent, players can extract "Engrams" (digitized skills and experiences) from specialized high-level Pops to sell on the Galactic Market for massive wealth. The extracted Pop, however, permanently loses those skills, suffers a severe "Hollowed" morale penalty, and their social relationships are wiped clean. The immediate financial gain is enormous, but the long-term cost is crippling your most valuable human resources.

## Dependencies

- Requires Pop Skills/Traits from `src/layer1/traits.rs`.
- Requires `Morale` from `src/layer1/social/morale.rs`.
- Requires `ColonyResources` or a wealth mechanic to reflect the galactic market sale.

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::traits::PopTraits;
    use crate::layer1::social::morale::Morale;
    use crate::shared::resources::ColonyResources;

    #[test]
    fn test_engram_extraction_removes_skills_and_grants_wealth() {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.add_systems(Update, process_engram_extraction);

        let initial_skills = PopTraits { engineering: 90.0, science: 85.0, ..default() };
        let entity = app.world_mut().spawn((
            initial_skills,
            Morale { value: 100.0 },
            EngramExtractionTarget,
        )).id();

        // Act
        app.update();

        // Assert: Skills are gone, wealth increased, Hollowed trait added
        let traits = app.world().get::<PopTraits>(entity).unwrap();
        assert_eq!(traits.engineering, 0.0);
        assert_eq!(traits.science, 0.0);

        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.wealth > 0.0); // Wealth should jump

        assert!(app.world().get::<HollowedState>(entity).is_some());
    }

    #[test]
    fn test_hollowed_morale_penalty() {
        let mut app = App::new();
        app.add_systems(Update, apply_hollowed_penalty);

        let entity = app.world_mut().spawn((
            HollowedState,
            Morale { value: 50.0 },
        )).id();

        // Act
        app.update();

        // Assert: Morale is capped or heavily drained
        let morale = app.world().get::<Morale>(entity).unwrap();
        assert!(morale.value < 20.0); // Severely punished
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::traits::PopTraits;
use crate::layer1::social::morale::Morale;
use crate::shared::resources::ColonyResources;

#[derive(Component)]
pub struct EngramExtractionTarget;

#[derive(Component)]
pub struct HollowedState;

pub fn process_engram_extraction(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(Entity, &mut PopTraits), With<EngramExtractionTarget>>,
) {
    for (entity, mut traits) in query.iter_mut() {
        // Calculate the "value" of the engram based on their highest skills
        let total_value = traits.engineering + traits.science + traits.agriculture; // Add more as needed

        // Grant the wealth
        resources.wealth += total_value * 10.0;

        // Wipe the pop
        traits.engineering = 0.0;
        traits.science = 0.0;
        traits.agriculture = 0.0;

        commands.entity(entity).remove::<EngramExtractionTarget>();
        commands.entity(entity).insert(HollowedState);
        // Note: Relationship wiping would happen here depending on relationship architecture
    }
}

pub fn apply_hollowed_penalty(
    mut query: Query<&mut Morale, With<HollowedState>>,
) {
    for mut morale in query.iter_mut() {
        // Hollowed pops have a permanent, massive drain or cap
        morale.value = morale.value.min(10.0);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Code Smell**: Hardcoding the value multiplier (`* 10.0`) and the skills wiped. A `TraitIterator` or a helper method on `PopTraits` like `wipe_skills()` returning a `f32` total value would be much cleaner.
- **Edge Case**: `HollowedState` currently hard-caps morale every frame. It might be better implemented as a negative aura or a massive baseline offset during the `evaluate_morale` system instead of overwriting it post-evaluation.
- **Lore Integration**: Extracting an engram should absolutely throw an `AddChronicleEvent` to document the sacrifice of the pop's mind for the empire's survival.

## Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Targeting a pop for engram extraction zeros their primary skills.
- [ ] Extracted skills are converted into `ColonyResources.wealth` (or equivalent currency).
- [ ] The pop receives a `HollowedState` that heavily penalizes or caps their Morale.

## Technical Guidance
- Create this in `src/layer1/social/engram.rs` or a similarly themed module.
- The `ColonyResources` modification must be mindful of how wealth is structured in the current `shared/resources.rs` file. If `wealth` isn't a field, use `credits` or `rare_materials`.
- Ensure you dispatch a chronicle event detailing the name of the Pop whose mind was wiped.

## Questions
*Builder: add questions here if spec is unclear.*
