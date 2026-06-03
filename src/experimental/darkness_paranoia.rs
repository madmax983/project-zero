//! Darkness Paranoia (Nova Feature).
//!
//! # The Spark
//! We have a `LightMap` representing illumination levels, and a `StressTracker` for Pops.
//! We also have `ActionType::Explore` and `ActionType::Idle`.
//!
//! # The Feature
//! Pops wandering around or idling in pitch darkness (`light < 0.1`) rapidly gain stress.
//! If the Pop has the `Trait::Anxious`, the stress accumulation is severely amplified.
//!
//! # The Potential
//! This connects the environment's `LightMap` with psychological health (`StressTracker`),
//! punishing players who fail to properly light their base's corridors and forcing them
//! to think about pathing lighting, not just workspace lighting.

use crate::layer1::lighting::LightMap;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;

/// System that adds stress to pops wandering or idling in darkness.
pub fn darkness_paranoia_system(
    light_map: Option<Res<LightMap>>,
    mut pops: Query<
        (
            &GridPosition,
            &PopAction,
            &mut StressTracker,
            Option<&Traits>,
        ),
        With<Pop>,
    >,
) {
    if let Some(light_map) = light_map {
        for (pos, action, mut stress, traits) in pops.iter_mut() {
            // Only affect idle or exploring pops
            if action.current == ActionType::Idle || action.current == ActionType::Explore {
                let x = u32::try_from(pos.x).unwrap_or(0);
                let y = u32::try_from(pos.y).unwrap_or(0);

                let light = light_map.get(x, y);

                // If it's very dark
                if light < 0.1 {
                    let mut penalty = 0.5; // Base stress penalty

                    if let Some(t) = traits {
                        if t.has(Trait::Anxious) {
                            penalty *= 3.0; // Anxious pops hate the dark
                        }
                    }

                    stress.accumulated_stress = (stress.accumulated_stress + penalty).min(100.0);
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(darkness_paranoia_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_darkness_paranoia_increases_stress() {
        let mut world = World::new();

        // Very dark map
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 0.0);
        world.insert_resource(light_map);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Idle,
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(darkness_paranoia_system).unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress > 0.0);
    }

    #[test]
    fn test_darkness_paranoia_amplified_for_anxious() {
        let mut world = World::new();

        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 0.0);
        world.insert_resource(light_map);

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Anxious);
            t
        };

        let anxious_pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Idle,
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                },
                traits,
            ))
            .id();

        let normal_pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Idle,
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(darkness_paranoia_system).unwrap();

        let stress_anxious = world
            .get::<StressTracker>(anxious_pop)
            .unwrap()
            .accumulated_stress;
        let stress_normal = world
            .get::<StressTracker>(normal_pop)
            .unwrap()
            .accumulated_stress;

        assert!(stress_anxious > stress_normal);
    }
}
