//! The Necro-Industrial Complex (Nova Feature).
//!
//! # The Spark
//! The colony faces constant threats, leading to casualties. We have `Corpse` entities
//! that cause grief, and `ColonyResources` that are always in demand. What if we could
//! industrialize the dead to fuel our grand stellar expansion?
//!
//! # The Feature
//! The `BiomassSublimator` building component processes `Corpse` entities within its radius.
//! When a corpse is sublimated, it yields high-quality organic slurry (`food`) and rare
//! elements (`metal`) essential for construction.
//! However, this horrifying, industrialized harvesting of loved ones causes severe psychological
//! damage, inflicting a massive `accumulated_stress` penalty on all living Pops nearby.
//!
//! # The Tension
//! The immense, immediate material value of treating the dead as a pure resource vs.
//! the profound social, psychological, and stability damage it inflicts on the living.

use crate::layer1::funeral::Corpse;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

/// A building component that sublimates corpses into resources.
#[derive(Component, Debug, Clone, Default)]
pub struct BiomassSublimator;

/// How far the sublimator reaches to grab corpses.
const SUBLIMATION_RADIUS: u32 = 2;
/// How far the psychological horror of the machine radiates.
const HORROR_RADIUS: u32 = 15;
/// The amount of food generated per corpse.
const FOOD_YIELD: f32 = 10.0;
/// The amount of metal generated per corpse.
const METAL_YIELD: f32 = 5.0;
/// The amount of stress inflicted on nearby Pops per corpse sublimated.
const STRESS_PENALTY: f32 = 25.0;

/// System that processes corpses near Biomass Sublimators into resources,
/// while inflicting massive stress on nearby Pops.
pub fn necro_industry_system(
    mut commands: Commands,
    sublimators: Query<&GridPosition, With<BiomassSublimator>>,
    corpses: Query<(Entity, &GridPosition), With<Corpse>>,
    mut pops: Query<(&GridPosition, &mut StressTracker), With<Pop>>,
    mut resources: ResMut<ColonyResources>,
) {
    let mut sublimated_count = 0;

    // Iterate over all corpses
    for (corpse_entity, corpse_pos) in corpses.iter() {
        // Check if near any sublimator
        let mut near_sublimator = false;
        for sublimator_pos in sublimators.iter() {
            // Fix: distance_chebyshev takes a value, so we deref
            if corpse_pos.distance_chebyshev(*sublimator_pos) <= SUBLIMATION_RADIUS {
                near_sublimator = true;
                break;
            }
        }

        if near_sublimator {
            // Consume the corpse
            commands.entity(corpse_entity).despawn();

            // Yield resources
            resources.food += FOOD_YIELD;
            resources.metal += METAL_YIELD;

            sublimated_count += 1;

            // Apply stress penalty to nearby pops
            for (pop_pos, mut stress) in pops.iter_mut() {
                if pop_pos.distance_chebyshev(*corpse_pos) <= HORROR_RADIUS {
                    stress.accumulated_stress += STRESS_PENALTY;
                }
            }
        }
    }

    if sublimated_count > 0 {
        log::warn!(
            "Necro-Industry: {} corpse(s) sublimated into resources. The colony shudders.",
            sublimated_count
        );
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(necro_industry_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world
    }

    #[test]
    fn test_sublimate_corpses_yields_resources_and_stress() {
        let mut world = setup_world();

        // Spawn Biomass Sublimator
        world.spawn((BiomassSublimator, GridPosition { x: 5, y: 5 }));

        // Spawn a Corpse nearby
        let corpse = world
            .spawn((
                Corpse {
                    name: "Unlucky Miner".to_string(),
                    decay: 0.0,
                },
                GridPosition { x: 6, y: 5 }, // Within radius 2
            ))
            .id();

        // Spawn a Pop nearby (within horror radius)
        let close_pop = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 5 }, // Distance 4 from corpse
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        // Spawn a Pop far away (outside horror radius)
        let far_pop = world
            .spawn((
                Pop,
                GridPosition { x: 30, y: 30 }, // Distance > 15
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(necro_industry_system).unwrap();

        // Verify Corpse is despawned
        assert!(
            world.get_entity(corpse).is_err(),
            "Corpse should be despawned"
        );

        // Verify Resources increased
        let resources = world.resource::<ColonyResources>();
        // Default food is 10.0
        assert_eq!(resources.food, 10.0 + FOOD_YIELD, "Food should increase");
        assert_eq!(resources.metal, 0.0 + METAL_YIELD, "Metal should increase");

        // Verify nearby Pop gained stress
        let close_stress = world.get::<StressTracker>(close_pop).unwrap();
        assert_eq!(
            close_stress.accumulated_stress, STRESS_PENALTY,
            "Nearby Pop should gain stress"
        );

        // Verify far Pop gained no stress
        let far_stress = world.get::<StressTracker>(far_pop).unwrap();
        assert_eq!(
            far_stress.accumulated_stress, 0.0,
            "Far Pop should not gain stress"
        );
    }

    #[test]
    fn test_corpses_outside_radius_are_ignored() {
        let mut world = setup_world();

        // Spawn Biomass Sublimator
        world.spawn((BiomassSublimator, GridPosition { x: 5, y: 5 }));

        // Spawn a Corpse far away
        let corpse = world
            .spawn((
                Corpse {
                    name: "Distant Miner".to_string(),
                    decay: 0.0,
                },
                GridPosition { x: 20, y: 20 }, // Outside radius 2
            ))
            .id();

        world.run_system_once(necro_industry_system).unwrap();

        // Verify Corpse is NOT despawned
        assert!(
            world.get_entity(corpse).is_ok(),
            "Corpse should not be despawned"
        );

        // Verify Resources did NOT increase
        let resources = world.resource::<ColonyResources>();
        assert_eq!(
            resources.food, 10.0,
            "Food should not increase from default"
        );
    }
}
