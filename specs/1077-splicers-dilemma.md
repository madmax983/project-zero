# 1077 Splicer's Dilemma

## 1. Overview
The "Splicer's Dilemma" introduces genetic clinics that allow players to radically alter Pops to thrive in extreme environments (e.g., radioactive areas, extreme heat). However, highly modified Pops lose the "Human" trait and gain a "Mutant" trait. This mechanic introduces a new axis of tension: base-human Pops develop "Xenophobia" towards Mutants, and heavily mutated Pops refuse to use standard human amenities or eat normal food. This creates physical segregation and potential civil wars.

## 2. Dependencies
- Layer 1 Resource System
- Layer 1 Needs System (Food, Amenities)
- Layer 1 Pop Traits (`Human`, `Mutant`, `Xenophobic`)
- Social Interactions System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Traits};
    use crate::layer1::needs::{FoodPreference, AmenityPreference};
    use crate::layer1::social::SocialRelation;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Add systems under test
        app.add_systems(Update, apply_mutation_system);
        app.add_systems(Update, evaluate_social_friction_system);
        app
    }

    #[test]
    fn test_mutation_removes_human_adds_mutant() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Traits(vec!["Human".to_string()]),
            MutationTarget { target_level: 100.0 }, // Trigger high mutation
        )).id();

        app.update();

        let traits = app.world().get::<Traits>(pop).unwrap();
        assert!(!traits.0.contains(&"Human".to_string()));
        assert!(traits.0.contains(&"Mutant".to_string()));
    }

    #[test]
    fn test_mutants_change_preferences() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Traits(vec!["Mutant".to_string()]),
            FoodPreference::Standard,
            AmenityPreference::Standard,
        )).id();

        app.update();

        let food_pref = app.world().get::<FoodPreference>(pop).unwrap();
        let amenity_pref = app.world().get::<AmenityPreference>(pop).unwrap();

        assert_eq!(*food_pref, FoodPreference::MutantSpecific);
        assert_eq!(*amenity_pref, AmenityPreference::MutantSpecific);
    }

    #[test]
    fn test_human_mutant_social_friction() {
        let mut app = setup_app();

        let human_pop = app.world_mut().spawn((
            Pop,
            Traits(vec!["Human".to_string()]),
        )).id();

        let mutant_pop = app.world_mut().spawn((
            Pop,
            Traits(vec!["Mutant".to_string()]),
        )).id();

        // Simulate social interaction
        app.world_mut().spawn(SocialInteraction {
            initiator: human_pop,
            target: mutant_pop,
        });

        app.update();

        // Check relationship standing
        // Assuming we have a way to lookup relations
        // Let's assume a resource or component tracking this
        // For this test, we expect the human to gain 'Xenophobic' trait or lose relationship standing
        let human_traits = app.world().get::<Traits>(human_pop).unwrap();
        assert!(human_traits.0.contains(&"Xenophobic".to_string()));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Traits(pub Vec<String>);

#[derive(Component)]
pub struct MutationTarget {
    pub target_level: f32,
}

#[derive(Component, PartialEq, Eq, Debug)]
pub enum FoodPreference {
    Standard,
    MutantSpecific,
}

#[derive(Component, PartialEq, Eq, Debug)]
pub enum AmenityPreference {
    Standard,
    MutantSpecific,
}

#[derive(Component)]
pub struct SocialInteraction {
    pub initiator: Entity,
    pub target: Entity,
}

pub fn apply_mutation_system(
    mut query: Query<(&mut Traits, &MutationTarget, &mut FoodPreference, &mut AmenityPreference), With<Pop>>,
) {
    for (mut traits, target, mut food, mut amenity) in query.iter_mut() {
        if target.target_level >= 100.0 {
            traits.0.retain(|t| t != "Human");
            if !traits.0.contains(&"Mutant".to_string()) {
                traits.0.push("Mutant".to_string());
            }
            *food = FoodPreference::MutantSpecific;
            *amenity = AmenityPreference::MutantSpecific;
        }
    }
}

pub fn evaluate_social_friction_system(
    mut commands: Commands,
    interactions: Query<(Entity, &SocialInteraction)>,
    mut traits_query: Query<&mut Traits>,
) {
    for (entity, interaction) in interactions.iter() {
        if let Ok(target_traits) = traits_query.get(interaction.target) {
            if target_traits.0.contains(&"Mutant".to_string()) {
                if let Ok(mut initiator_traits) = traits_query.get_mut(interaction.initiator) {
                    if initiator_traits.0.contains(&"Human".to_string()) {
                        if !initiator_traits.0.contains(&"Xenophobic".to_string()) {
                            initiator_traits.0.push("Xenophobic".to_string());
                        }
                    }
                }
            }
        }
        commands.entity(entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Trait Implementation:** Convert `Traits(Vec<String>)` into a `HashSet` or enum-based system for O(1) lookups and memory efficiency.
- **Gradual Mutation:** Instead of instant mutation at 100.0, introduce a `MutationLevel` component that gradually increments over time. Introduce intermediate tiers of mutation (e.g., "Slightly Mutated", "Severely Mutated").
- **Preferences:** Generalize `FoodPreference` and `AmenityPreference` into a robust tagging system where Pops require items/buildings matching their tags.
- **Social Friction:** Make the Xenophobia acquisition stochastic or based on prolonged exposure rather than a single interaction.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops reaching critical mutation levels swap Human traits for Mutant traits.
- [ ] Mutant Pops switch to mutant-specific food and amenity needs.
- [ ] Humans interacting with Mutants gain the Xenophobic trait.

## 7. Technical Guidance
- The social friction system should plug into the existing `social_system` and utility AI.
- Ensure that Mutant Pops can still perform their assigned jobs despite changing preferences.

## 8. Questions
*Builder: add questions here if spec is unclear.*
