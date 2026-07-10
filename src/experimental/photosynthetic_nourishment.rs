//! Photosynthetic Nourishment system.
//!
//! Connects `Trait::Photosynthesis` with `LightMap` and `AmbientLight`.
//! Pops with the Photosynthesis trait will passively regenerate their hunger need
//! when exposed to high light levels, turning them into efficient daytime workers
//! that require little food.

use crate::layer1::lighting::{AmbientLight, LightMap};
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// If a pop has Photosynthesis and is exposed to light > this threshold, they regenerate hunger.
const PHOTOSYNTHESIS_LIGHT_THRESHOLD: f32 = 0.5;
/// How much hunger is regenerated per tick when photosynthesizing.
const PHOTOSYNTHESIS_NOURISHMENT_RATE: f32 = 0.002;

/// Checks for pops with the Photosynthesis trait and regenerates their hunger
/// based on the light level at their position.
pub fn photosynthetic_nourishment_system(
    light_map: Res<LightMap>,
    ambient_light: Res<AmbientLight>,
    mut pop_query: Query<(&GridPosition, &Traits, &mut Needs), With<Pop>>,
) {
    for (pos, traits, mut needs) in pop_query.iter_mut() {
        if traits.has(Trait::Photosynthesis) {
            // Check local light level plus ambient light to determine total light exposure.
            let local_light = light_map.get(pos.x as u32, pos.y as u32);
            let total_light = local_light.max(ambient_light.level);

            if total_light > PHOTOSYNTHESIS_LIGHT_THRESHOLD {
                needs.hunger = (needs.hunger + PHOTOSYNTHESIS_NOURISHMENT_RATE).min(1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_photosynthesis_nourishes_in_light() {
        let mut world = World::new();

        let mut map = LightMap::new(10, 10);
        map.set(5, 5, 1.0); // Bright light at (5, 5)
        world.insert_resource(map);
        world.insert_resource(AmbientLight { level: 0.0 });

        let mut traits = Traits::default();
        traits.add(Trait::Photosynthesis);

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 0.5,
                    rest: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                },
            ))
            .id();

        let _ = world.run_system_once(photosynthetic_nourishment_system);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.hunger > 0.5,
            "Photosynthetic pop should gain hunger in light"
        );
    }

    #[test]
    fn test_photosynthesis_does_not_nourish_in_darkness() {
        let mut world = World::new();

        let map = LightMap::new(10, 10);
        world.insert_resource(map);
        world.insert_resource(AmbientLight { level: 0.0 }); // Pitch black

        let mut traits = Traits::default();
        traits.add(Trait::Photosynthesis);

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 0.5,
                    rest: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                },
            ))
            .id();

        let _ = world.run_system_once(photosynthetic_nourishment_system);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.hunger - 0.5).abs() < f32::EPSILON,
            "Photosynthetic pop should NOT gain hunger in darkness"
        );
    }

    #[test]
    fn test_non_photosynthetic_pops_unaffected() {
        let mut world = World::new();

        let mut map = LightMap::new(10, 10);
        map.set(5, 5, 1.0); // Bright light at (5, 5)
        world.insert_resource(map);
        world.insert_resource(AmbientLight { level: 0.0 });

        let traits = Traits::default(); // No photosynthesis

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 0.5,
                    rest: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                },
            ))
            .id();

        let _ = world.run_system_once(photosynthetic_nourishment_system);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.hunger - 0.5).abs() < f32::EPSILON,
            "Normal pop should NOT gain hunger from light"
        );
    }
}
