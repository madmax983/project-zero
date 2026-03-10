//! Psychic Resonance (Nova Feature).
//!
//! # The Spark
//! We have a `Morale` system and `Items` on the ground. What if some Pops are "Psychic" and
//! can passively sense the presence of valuable items (like Rations or Alcohol) through walls and over
//! long distances, giving them a morale boost just by being in the same general area as
//! hidden wealth?
//!
//! # The Feature
//! The `Psychic` trait allows a pop to resonate with `ResourceItem`s. If they are near
//! high-value items, they gain a slow, passive `Morale` increase, representing the "hum"
//! of potential. If they are near Waste, they lose morale. This adds a spatial puzzle
//! to base design: where do you store your wealth relative to your psychic pops?

use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use crate::layer1::resources::{ResourceItem, ResourceType};
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const SENSE_RADIUS: i32 = 15; // They can "sense" things quite far away
const RESONANCE_STRENGTH: f32 = 0.005;

pub fn psychic_resonance_system(
    mut pops: Query<(&GridPosition, &mut Morale, &Traits), With<Pop>>,
    items: Query<(&GridPosition, &ResourceItem)>,
) {
    for (pop_pos, mut morale, traits) in pops.iter_mut() {
        // We simulate a 'Psychic' trait using `Trait::VoidTouched`
        if !traits.0.contains(&Trait::VoidTouched) {
            continue;
        }

        let mut resonance_delta = 0.0;

        // Sense items
        for (item_pos, item) in items.iter() {
            let dist = pop_pos.distance_chebyshev(*item_pos);
            if dist <= SENSE_RADIUS as u32 {
                // High value items give positive resonance
                match item.resource_type {
                    ResourceType::Alcohol | ResourceType::Rations | ResourceType::Metal => {
                        resonance_delta += RESONANCE_STRENGTH * (item.amount as f32).min(10.0);
                    }
                    ResourceType::Waste | ResourceType::Scrap => {
                        resonance_delta -= RESONANCE_STRENGTH * (item.amount as f32).min(10.0);
                    }
                    _ => {}
                }
            }
        }

        if resonance_delta != 0.0 {
            morale.value = (morale.value + resonance_delta).clamp(0.0, 1.0);
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(psychic_resonance_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;
    use std::collections::HashSet;

    #[test]
    fn test_psychic_resonance_positive() {
        let mut world = World::new();

        let traits = Traits(HashSet::from([Trait::VoidTouched]));
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
                traits,
            ))
            .id();

        // Spawn an alcohol item nearby
        world.spawn((
            GridPosition { x: 5, y: 5 }, // Within radius 15
            ResourceItem {
                resource_type: ResourceType::Alcohol,
                amount: 10.0,
            },
        ));

        world.run_system_once(psychic_resonance_system).unwrap();

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(
            morale.value > 0.5,
            "Morale should increase due to positive psychic resonance"
        );
    }

    #[test]
    fn test_psychic_resonance_negative() {
        let mut world = World::new();

        let traits = Traits(HashSet::from([Trait::VoidTouched]));
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
                traits,
            ))
            .id();

        // Spawn a waste item nearby
        world.spawn((
            GridPosition { x: 5, y: 5 }, // Within radius 15
            ResourceItem {
                resource_type: ResourceType::Waste,
                amount: 10.0,
            },
        ));

        world.run_system_once(psychic_resonance_system).unwrap();

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(
            morale.value < 0.5,
            "Morale should decrease due to negative psychic resonance"
        );
    }

    #[test]
    fn test_non_psychic_ignores_resonance() {
        let mut world = World::new();

        let traits = Traits(HashSet::new()); // Not VoidTouched
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
                traits,
            ))
            .id();

        world.spawn((
            GridPosition { x: 5, y: 5 },
            ResourceItem {
                resource_type: ResourceType::Alcohol,
                amount: 10.0,
            },
        ));

        world.run_system_once(psychic_resonance_system).unwrap();

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(
            (morale.value - 0.5).abs() < f32::EPSILON,
            "Non-psychic pop should not be affected"
        );
    }
}
