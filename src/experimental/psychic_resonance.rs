//! Psychic Resonance (Nova Feature).
//!
//! # The Spark
//! We have a `Morale` system and `ResourceItem`s scattered or stockpiled. What if some Pops are "Psychic" and
//! can passively sense the presence of valuable items (like Rations or Alcohol) through walls and over
//! long distances, giving them a morale boost just by being in the same general area as
//! hidden wealth?
//!
//! # The Feature
//! The `Psychic` trait allows a pop to resonate with `ResourceItem`s. If they are near
//! high-value items, they gain a slow, passive `leisure` (and thus `Morale`) increase, representing the "hum"
//! of potential. If they are near Waste or Scrap, they lose leisure. This adds a spatial puzzle
//! to base design: where do you store your wealth relative to your psychic pops?

use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::resources::{ResourceItem, ResourceType};
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const SENSE_RADIUS: i32 = 15; // They can "sense" things quite far away
const RESONANCE_STRENGTH: f32 = 0.005;

pub fn psychic_resonance_system(
    mut pops: Query<(&GridPosition, &mut Needs, &Traits), With<Pop>>,
    items: Query<(&GridPosition, &ResourceItem)>,
) {
    for (pop_pos, mut needs, traits) in pops.iter_mut() {
        if !traits.has(Trait::VoidTouched) {
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
                        resonance_delta += RESONANCE_STRENGTH * item.amount.min(10.0);
                    }
                    ResourceType::Waste | ResourceType::Scrap => {
                        resonance_delta -= RESONANCE_STRENGTH * item.amount.min(10.0);
                    }
                    _ => {}
                }
            }
        }

        if resonance_delta != 0.0 {
            needs.leisure = (needs.leisure + resonance_delta).clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;
    use std::collections::HashSet;

    #[test]
    fn test_psychic_resonance_positive() {
        let mut world = World::new();

        let traits = Traits(1 << (Trait::VoidTouched as u8));
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 0.5,
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

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Leisure should increase due to positive psychic resonance"
        );
    }

    #[test]
    fn test_psychic_resonance_negative() {
        let mut world = World::new();

        let traits = Traits(1 << (Trait::VoidTouched as u8));
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 0.5,
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

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure < 0.5,
            "Leisure should decrease due to negative psychic resonance"
        );
    }

    #[test]
    fn test_non_psychic_ignores_resonance() {
        let mut world = World::new();

        let traits = Traits::default(); // Not VoidTouched
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 0.5,
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

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Non-psychic pop should not be affected"
        );
    }
}
