use bevy_app::prelude::*;
use scale::layer1::combat::{Armor, CombatStats, DamageType};
use scale::layer1::fauna::modular_fauna::{evaluate_modular_fauna_stats_system, AnimalParts, PartType};
use scale::layer1::fauna::Fauna;

#[test]
fn test_modular_fauna_stats_integration() {
    let mut app = App::new();

    // Instead of explicitly registering the full economy schedule (which expects specific resources to be present),
    // we can check if the system correctly functions alongside Update schedule, like it's done typically for isolated tests.
    // The main verification is that the system operates correctly on the world when present.
    app.add_systems(Update, evaluate_modular_fauna_stats_system);

    // Spawn a Chimera
    let chimera = app.world_mut().spawn((
        Fauna::default(),
        AnimalParts {
            head: PartType::WolfHead,
            body: PartType::BearTorso,
            limbs: PartType::CrabLegs,
            tail: PartType::ScorpionTail,
        },
        CombatStats::default(),
        Armor { rating: 0 },
    )).id();

    app.update();

    let stats = app.world().get::<CombatStats>(chimera).unwrap();
    let armor = app.world().get::<Armor>(chimera).unwrap();

    // Verify stats were aggregated from the specific parts
    assert!(stats.melee_damage > 10.0, "Wolf Head should add high melee bite damage. Got: {}", stats.melee_damage);
    assert!(stats.damage_types.contains(&DamageType::Venom), "Scorpion Tail should add Venom damage type.");
    assert!(armor.rating > 5, "Crab Legs should provide heavy armor rating. Got: {}", armor.rating);
}
