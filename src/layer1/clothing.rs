//! Clothing and Temperature system.
//!
//! Handles Hypothermia in Winter and Clothing degradation.

use crate::layer1::health::Health;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::seasons::{Season, SeasonState};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Applies hypothermia damage to pops without clothing during Winter.
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
    #[allow(clippy::cast_precision_loss)]
    let pop_count = pop_query.iter().count() as f32;

    if pop_count == 0.0 {
        return;
    }

    // Calculate shortage ratio (0.0 = full clothes, 1.0 = no clothes)
    // If clothing >= pop_count, shortage = 0.
    // If clothing = 0, shortage = 1.
    // If clothing = pop_count / 2, shortage = 0.5.
    let shortage = (1.0 - (clothing_available / pop_count)).clamp(0.0, 1.0);

    if shortage <= 0.0 {
        return;
    }

    let mut rng = rand::thread_rng();

    for mut health in &mut pop_query {
        // Probabilistic damage based on shortage
        if rng.gen_bool(f64::from(shortage)) {
            // Deal damage!
            health.take_damage(1.0);
        }
    }
}

/// Degrades clothing over time based on usage.
pub fn clothing_wear_system(pop_query: Query<&Pop>, mut resources: ResMut<ColonyResources>) {
    #[allow(clippy::cast_precision_loss)]
    let pop_count = pop_query.iter().count() as f32;
    let used_clothing = resources.clothing.min(pop_count);

    if used_clothing <= 0.0 {
        return;
    }

    // Simple decay: Fixed % chance per tick per used item
    // OR simplified: decay = used_clothing * RATE
    let decay_rate = 0.001;
    let amount_lost = used_clothing * decay_rate;

    if amount_lost > 0.0 {
        // Direct subtraction, clamp to 0
        resources.clothing = (resources.clothing - amount_lost).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::building::BuildingType;
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer1::refining::get_refining_recipe;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::seasons::{Season, SeasonState};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

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

        let (can_afford, input, output) = get_refining_recipe(BuildingType::Weaver, &res);

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

        let (can_afford, input, output) = get_refining_recipe(BuildingType::Tailor, &res);

        assert!(can_afford);
        assert_eq!(input.cloth, 1.0);
        assert_eq!(output.clothing, 1.0);
    }

    // 3. Hypothermia System Tests
    #[test]
    fn test_hypothermia_damage_in_winter_no_clothes() {
        let mut world = World::new();
        world.insert_resource(SeasonState {
            current_season: Season::Winter,
        });
        world.insert_resource(ColonyResources {
            clothing: 0.0,
            ..Default::default()
        });

        // Spawn Pop
        let pop = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Run system multiple times to ensure probabilistic damage hits eventually
        // Or mock the RNG if possible. For now, assume high probability or deterministic test loop.
        // With 1.0 shortage (0 clothes), chance is 100%.

        world.run_system_once(super::hypothermia_system).unwrap();

        let health = world.get::<Health>(pop).unwrap();
        assert!(
            health.current < 100.0,
            "Pop should take damage in winter with no clothes"
        );
    }

    #[test]
    fn test_clothing_prevents_hypothermia() {
        let mut world = World::new();
        world.insert_resource(SeasonState {
            current_season: Season::Winter,
        });
        // 1 Pop, 1 Clothing
        world.insert_resource(ColonyResources {
            clothing: 1.0,
            ..Default::default()
        });

        let pop = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        world.run_system_once(super::hypothermia_system).unwrap();

        let health = world.get::<Health>(pop).unwrap();
        assert_eq!(health.current, 100.0);
    }

    #[test]
    fn test_clothing_degradation() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            clothing: 10.0,
            ..Default::default()
        });
        // Spawn pops to use the clothing
        for _ in 0..10 {
            world.spawn(Pop);
        }

        // Run wear system
        world.run_system_once(super::clothing_wear_system).unwrap();

        let res = world.resource::<ColonyResources>();
        // Should be < 10.0.
        assert!(res.clothing < 10.0);
    }
}
