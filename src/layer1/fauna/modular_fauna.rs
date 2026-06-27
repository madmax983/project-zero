use crate::layer1::combat::{Armor, CombatStats, DamageType};
use crate::layer1::fauna::Fauna;
use bevy_ecs::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PartType {
    WolfHead,
    BunnyHead,
    BearTorso,
    CrabLegs,
    GazelleLegs,
    ScorpionTail,
}

#[derive(Component, Debug, Clone)]
pub struct AnimalParts {
    pub head: PartType,
    pub body: PartType,
    pub limbs: PartType,
    pub tail: PartType,
}

pub fn evaluate_modular_fauna_stats_system(
    mut query: Query<(&AnimalParts, &mut CombatStats, &mut Armor), With<Fauna>>,
) {
    for (parts, mut stats, mut armor) in query.iter_mut() {
        // Reset base
        stats.melee_damage = 5.0;
        stats.damage_types.clear();
        armor.rating = 0;

        // Evaluate Head
        if parts.head == PartType::WolfHead {
            stats.melee_damage += 15.0;
            stats.damage_types.push(DamageType::Piercing);
        } else if parts.head == PartType::BunnyHead {
            // Cute but weak
            stats.melee_damage += 1.0;
        }

        // Evaluate Limbs
        if parts.limbs == PartType::CrabLegs {
            armor.rating += 10; // Heavy plating
        } else if parts.limbs == PartType::GazelleLegs {
            // High speed (not tracked in this test/MVP but should be)
            armor.rating += 1;
        }

        // Evaluate Tail
        if parts.tail == PartType::ScorpionTail {
            stats.damage_types.push(DamageType::Venom);
            stats.melee_damage += 10.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_modular_parts_construct_combat_stats() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_modular_fauna_stats_system);

        let chimera = app
            .world_mut()
            .spawn((
                Fauna::default(),
                AnimalParts {
                    head: PartType::WolfHead,
                    body: PartType::BearTorso,
                    limbs: PartType::CrabLegs,
                    tail: PartType::ScorpionTail,
                },
                CombatStats::default(),
                Armor { rating: 0 },
            ))
            .id();

        app.update();

        let stats = app.world().get::<CombatStats>(chimera).unwrap();
        let armor = app.world().get::<Armor>(chimera).unwrap();

        // Verify stats were aggregated from the specific parts
        assert!(
            stats.melee_damage > 10.0,
            "Wolf Head should add high melee bite damage."
        );
        assert!(
            stats.damage_types.contains(&DamageType::Venom),
            "Scorpion Tail should add Venom damage type."
        );
        assert!(
            armor.rating > 5,
            "Crab Legs should provide heavy armor rating."
        );
    }
}
