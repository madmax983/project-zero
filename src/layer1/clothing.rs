//! Clothing and Temperature system.
//!
//! Handles Hypothermia in Winter and Clothing degradation.

use crate::layer1::health::Health;
use crate::layer1::items::{Clothing, Equipment};
use crate::layer1::pop::Pop;
// use crate::layer1::resources::ColonyResources; // Unused in new logic
use crate::layer1::seasons::{Season, SeasonState};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Applies hypothermia damage to pops without clothing during Winter.
pub fn hypothermia_system(
    mut pop_query: Query<(&mut Health, &Equipment), With<Pop>>,
    clothing_query: Query<&Clothing>,
    season: Option<Res<SeasonState>>,
) {
    // Only applies in Winter
    if !matches!(season.map(|s| s.current_season), Some(Season::Winter)) {
        return;
    }

    let mut rng = rand::thread_rng();

    for (mut health, equipment) in &mut pop_query {
        let mut insulation = 0.0;

        if let Some(item) = equipment.body.and_then(|e| clothing_query.get(e).ok()) {
            insulation += item.insulation;
        }

        // Damage chance = 1.0 - insulation
        // If insulation is 1.0 (Full), chance is 0.
        // If insulation is 0.0 (Naked), chance is 100%.
        let damage_chance = (1.0 - insulation).clamp(0.0, 1.0);

        if rng.gen_bool(f64::from(damage_chance)) {
            health.take_damage(1.0);
        }
    }
}

/// Degrades clothing over time based on usage.
pub fn clothing_wear_system(
    mut commands: Commands,
    mut pop_query: Query<&mut Equipment, With<Pop>>,
    mut clothing_query: Query<&mut Clothing>,
) {
    let decay_amount = 0.05; // Tunable

    for mut eq in &mut pop_query {
        if let Some(entity) = eq.body {
            if let Ok(mut item) = clothing_query.get_mut(entity) {
                item.durability -= decay_amount;

                if item.durability <= 0.0 {
                    // Break item
                    commands.entity(entity).despawn();
                    eq.body = None;
                }
            } else {
                // Entity missing, clear slot
                eq.body = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::items::{Item, Clothing, ClothingType, Equipment};
    use crate::layer1::health::Health;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::seasons::{Season, SeasonState};
    use super::{hypothermia_system, clothing_wear_system};
    use bevy_ecs::system::RunSystemOnce;

    // 1. Equipment Slots
    #[test]
    fn test_equipment_has_body_slot() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Equipment::default())).id();
        let eq = world.get::<Equipment>(entity).unwrap();

        // Assert new fields exist (compiler check mostly, but good for TDD)
        assert!(eq.body.is_none());
        assert!(eq.head.is_none());
    }

    // 2. Clothing Component
    #[test]
    fn test_clothing_component() {
        let mut world = World::new();
        let tunic = world.spawn((
            Item,
            Clothing {
                clothing_type: ClothingType::Tunic,
                insulation: 1.0,
                durability: 100.0,
                max_durability: 100.0,
            }
        )).id();

        let c = world.get::<Clothing>(tunic).unwrap();
        assert_eq!(c.insulation, 1.0);
    }

    // 3. Hypothermia Logic (Refactored)
    #[test]
    fn test_hypothermia_checks_equipment() {
        let mut world = World::new();
        world.insert_resource(SeasonState { current_season: Season::Winter });
        // Global resource should be ignored or used only for "available" count,
        // but damage depends on Equipment.
        world.insert_resource(ColonyResources::default());

        // Pop 1: Naked (Should take damage)
        let pop1 = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            Equipment::default(), // No body
        )).id();

        // Pop 2: Clothed (Should be safe)
        let tunic = world.spawn(Clothing {
            clothing_type: ClothingType::Tunic,
            insulation: 1.0,
            durability: 100.0,
            max_durability: 100.0,
        }).id();

        let pop2 = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            Equipment { body: Some(tunic), ..Default::default() },
        )).id();

        // Run system
        // With logic implemented:
        // Pop 1 (naked) -> 100% chance -> takes 1.0 damage.
        // Pop 2 (insulation 1.0) -> 0% chance -> takes 0 damage.

        world.run_system_once(hypothermia_system).unwrap();

        let h1 = world.get::<Health>(pop1).unwrap();
        let h2 = world.get::<Health>(pop2).unwrap();

        assert!(h1.current < 100.0, "Naked pop should freeze");
        assert_eq!(h2.current, 100.0, "Clothed pop should be warm");
    }

    // 4. Wear Logic
    #[test]
    fn test_clothing_degrades_on_wearer() {
        let mut world = World::new();

        let tunic = world.spawn(Clothing {
            clothing_type: ClothingType::Tunic,
            insulation: 1.0,
            durability: 10.0,
            max_durability: 100.0,
        }).id();

        let _pop = world.spawn((
            Pop,
            Equipment { body: Some(tunic), ..Default::default() },
        )).id();

        world.run_system_once(clothing_wear_system).unwrap();

        let c = world.get::<Clothing>(tunic).unwrap();
        // Decay amount is 0.05
        assert!(c.durability < 10.0);
        assert!((c.durability - 9.95).abs() < 0.001);
    }

    // 5. Breakage
    #[test]
    fn test_clothing_breaks() {
        let mut world = World::new();

        let tunic = world.spawn(Clothing {
            clothing_type: ClothingType::Tunic,
            insulation: 1.0,
            durability: 0.001, // Almost broken
            max_durability: 100.0,
        }).id();

        let pop = world.spawn((
            Pop,
            Equipment { body: Some(tunic), ..Default::default() },
        )).id();

        // Should break and despawn
        world.run_system_once(clothing_wear_system).unwrap();

        // Entity should be despawned
        assert!(world.get_entity(tunic).is_err());

        // Slot should be None
        let eq = world.get::<Equipment>(pop).unwrap();
        assert!(eq.body.is_none());
    }
}
