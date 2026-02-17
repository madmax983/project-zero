//! Clothing and Temperature system.
//!
//! Handles Hypothermia in Winter and Clothing degradation.

use crate::layer1::items::{Clothing, Equipment};
use crate::layer1::pop::Pop;
// use crate::layer1::resources::ColonyResources; // Unused in new logic
use bevy_ecs::prelude::*;

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
                Item,
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
}
