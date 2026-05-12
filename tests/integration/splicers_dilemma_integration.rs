use bevy::prelude::*;
use scale::layer1::biology::genetics::splicers_dilemma::{apply_mutation_system, evaluate_social_friction_system, MutationTarget, FoodPreference, AmenityPreference, SocialInteraction};
use scale::layer1::entities::pop::Pop;
use scale::layer1::psychology::traits::{Trait, Traits};

#[test]
fn test_splicers_dilemma_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (apply_mutation_system, evaluate_social_friction_system).chain());

    let mutant_pop = app.world_mut().spawn((
        Pop,
        Traits::default(),
        MutationTarget { target_level: 100.0 },
        FoodPreference::Standard,
        AmenityPreference::Standard,
    )).id();

    let human_pop = app.world_mut().spawn((
        Pop,
        Traits::default(),
    )).id();

    app.world_mut().spawn(SocialInteraction {
        initiator: human_pop,
        target: mutant_pop,
    });

    app.update();

    let traits = app.world().get::<Traits>(mutant_pop).unwrap();
    assert!(traits.has(Trait::Mutant));
    let food_pref = app.world().get::<FoodPreference>(mutant_pop).unwrap();
    assert_eq!(*food_pref, FoodPreference::MutantSpecific);

    let human_traits = app.world().get::<Traits>(human_pop).unwrap();
    assert!(human_traits.has(Trait::Xenophobic));
}
