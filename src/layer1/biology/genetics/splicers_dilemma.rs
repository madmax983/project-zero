use bevy::prelude::*;
use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::traits::{Trait, Traits};

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
    mut query: Query<
        (
            &mut Traits,
            &MutationTarget,
            &mut FoodPreference,
            &mut AmenityPreference,
        ),
        With<Pop>,
    >,
) {
    for (mut traits, target, mut food, mut amenity) in query.iter_mut() {
        if target.target_level >= 100.0 {
            traits.0.insert(Trait::Mutant);
            *food = FoodPreference::MutantSpecific;
            *amenity = AmenityPreference::MutantSpecific;
        } else if traits.has(Trait::Mutant) {
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
        let mut should_add_xenophobic = false;
        if let Ok(target_traits) = traits_query.get(interaction.target) {
            if target_traits.has(Trait::Mutant) {
                should_add_xenophobic = true;
            }
        }

        if should_add_xenophobic {
            if let Ok(mut initiator_traits) = traits_query.get_mut(interaction.initiator) {
                if !initiator_traits.has(Trait::Mutant) {
                    initiator_traits.0.insert(Trait::Xenophobic);
                }
            }
        }
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::traits::{Trait, Traits};

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

        // In our codebase, base human pops just have default Traits (or lacking Mutant)
        let pop = app.world_mut().spawn((
            Pop,
            Traits::default(),
            MutationTarget { target_level: 100.0 },
            FoodPreference::Standard,
            AmenityPreference::Standard,
        )).id();

        app.update();

        let traits = app.world().get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::Mutant));
    }

    #[test]
    fn test_mutants_change_preferences() {
        let mut app = setup_app();

        let mut t = Traits::default();
        t.0.insert(Trait::Mutant);

        let pop = app.world_mut().spawn((
            Pop,
            t,
            MutationTarget { target_level: 0.0 }, // Include MutationTarget so the query matches!
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
            Traits::default(),
        )).id();

        let mut t = Traits::default();
        t.0.insert(Trait::Mutant);

        let mutant_pop = app.world_mut().spawn((
            Pop,
            t,
        )).id();

        app.world_mut().spawn(SocialInteraction {
            initiator: human_pop,
            target: mutant_pop,
        });

        app.update();

        let human_traits = app.world().get::<Traits>(human_pop).unwrap();
        assert!(human_traits.has(Trait::Xenophobic));
    }
}
