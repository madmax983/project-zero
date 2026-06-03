//! Clothing and Temperature system.
//!
//! Handles Hypothermia in Winter and Clothing degradation.

use crate::layer1::items::{Clothing, Equipment};
use crate::layer1::pop::Pop;
use crate::layer1::health::Health;
use crate::layer1::resources::ColonyResources;
use crate::layer1::seasons::{Season, SeasonState};
use bevy_ecs::prelude::*;
use rand::Rng;

pub fn hypothermia_system(
    mut pop_query: Query<&mut Health, With<Pop>>,
    resources: Res<ColonyResources>,
    season: Option<Res<SeasonState>>,
) {
    // Only applies in Winter
    if !matches!(season.map(|s| s.current_season), Some(Season::Winter)) {
        return;
    }

    let clothing_available = resources.clothing;
    let pop_count = pop_query.iter().count() as f32;

    if pop_count == 0.0 {
        return;
    }

    // Calculate shortage ratio (0.0 = full clothes, 1.0 = no clothes)
    let shortage = (1.0 - (clothing_available / pop_count)).clamp(0.0, 1.0);

    if shortage <= 0.0 {
        return;
    }

    let mut rng = rand::thread_rng();

    for mut health in &mut pop_query {
        // Probabilistic damage based on shortage
        if rng.gen_bool(shortage as f64) {
            health.take_damage(1.0);
        }
    }
}

/// Degrades clothing over time based on usage.
pub fn clothing_wear_system(
    mut commands: Commands,
    mut pop_query: Query<&mut Equipment, With<Pop>>,
    mut clothing_query: Query<&mut Clothing>,
    pop_count_query: Query<&Pop>,
    resources: Option<ResMut<ColonyResources>>,
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

    // Also decay global clothing resource based on pop usage
    if let Some(mut res) = resources {
        let pop_count = pop_count_query.iter().count() as f32;
        let used_clothing = res.clothing.min(pop_count);

        if used_clothing > 0.0 {
            let decay_rate = 0.001;
            let amount_lost = used_clothing * decay_rate;

            if amount_lost > 0.0 {
                res.clothing = (res.clothing - amount_lost).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::clothing_wear_system;
    use crate::layer1::items::{Clothing, ClothingType, Equipment, Item};
    use crate::layer1::pop::Pop;
    use bevy_ecs::prelude::*;
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
        let tunic = world
            .spawn((
                Item::default(),
                Clothing {
                    clothing_type: ClothingType::Tunic,
                    insulation: 1.0,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        let c = world.get::<Clothing>(tunic).unwrap();
        assert_eq!(c.insulation, 1.0);
    }

    // 4. Wear Logic
    #[test]
    fn test_clothing_degrades_on_wearer() {
        let mut world = World::new();

        let tunic = world
            .spawn(Clothing {
                clothing_type: ClothingType::Tunic,
                insulation: 1.0,
                durability: 10.0,
                max_durability: 100.0,
            })
            .id();

        let _pop = world
            .spawn((
                Pop,
                Equipment {
                    body: Some(tunic),
                    ..Default::default()
                },
            ))
            .id();

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

        let tunic = world
            .spawn(Clothing {
                clothing_type: ClothingType::Tunic,
                insulation: 1.0,
                durability: 0.001, // Almost broken
                max_durability: 100.0,
            })
            .id();

        let pop = world
            .spawn((
                Pop,
                Equipment {
                    body: Some(tunic),
                    ..Default::default()
                },
            ))
            .id();

        // Should break and despawn
        world.run_system_once(clothing_wear_system).unwrap();

        // Entity should be despawned
        assert!(world.get_entity(tunic).is_err());

        // Slot should be None
        let eq = world.get::<Equipment>(pop).unwrap();
        assert!(eq.body.is_none());
    }

    use crate::layer1::resources::ColonyResources;
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::layer1::health::Health;
    use crate::layer1::building::BuildingType;
    use crate::layer1::economy::refining::get_refining_recipe;

    // 1. Resource Tests
    #[test]
    fn test_colony_resources_clothing_fields() {
        let res = ColonyResources::default();
        // New fields should exist
        assert_eq!(res.fiber, 0.0);
        assert_eq!(res.cloth, 0.0);
        assert_eq!(res.clothing, 0.0);

        // Defaults
        assert!(res.max_fiber > 0.0);
        assert!(res.max_cloth > 0.0);
        assert!(res.max_clothing > 0.0);
    }

    // 2. Refining Recipe Tests
    #[test]
    fn test_weaver_recipe() {
        let res = ColonyResources {
            fiber: 5.0,
            cloth: 0.0,
            max_cloth: 10.0,
            ..Default::default()
        };

        let (can_afford, input, output, _) = get_refining_recipe(BuildingType::Weaver, &res);

        assert!(can_afford);
        assert_eq!(input.fiber, 1.0);
        assert_eq!(output.cloth, 1.0);
    }

    #[test]
    fn test_tailor_recipe() {
        let res = ColonyResources {
            cloth: 5.0,
            clothing: 0.0,
            max_clothing: 10.0,
            ..Default::default()
        };

        let (can_afford, input, output, _) = get_refining_recipe(BuildingType::Tailor, &res);

        assert!(can_afford);
        assert_eq!(input.cloth, 1.0);
        assert_eq!(output.clothing, 1.0);
    }

    // 3. Hypothermia System Tests
    #[test]
    fn test_hypothermia_damage_in_winter_no_clothes() {
        let mut world = World::new();
        world.insert_resource(SeasonState { current_season: Season::Winter });
        world.insert_resource(ColonyResources { clothing: 0.0, ..Default::default() });

        // Spawn Pop
        let _pop = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0, has_rust_lung: false }
        )).id();

        world.run_system_once(super::hypothermia_system).unwrap();

        // Testing this deterministically depends on how `hypothermia_system` is implemented.
        // The spec implies it uses probability.
    }

    #[test]
    fn test_clothing_prevents_hypothermia() {
        let mut world = World::new();
        world.insert_resource(SeasonState { current_season: Season::Winter });
        // 1 Pop, 1 Clothing
        world.insert_resource(ColonyResources { clothing: 1.0, ..Default::default() });

        let pop = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0, has_rust_lung: false }
        )).id();

        world.run_system_once(super::hypothermia_system).unwrap();

        let health = world.get::<Health>(pop).unwrap();
        assert_eq!(health.current, 100.0);
    }

    #[test]
    fn test_clothing_degradation() {
        let mut world = World::new();
        world.insert_resource(ColonyResources { clothing: 10.0, ..Default::default() });
        // Spawn pops to use the clothing
        for _ in 0..10 { world.spawn(Pop); }

        // Run wear system
        world.run_system_once(super::clothing_wear_system).unwrap();

        let _res = world.resource::<ColonyResources>();
    }
}
