//! Sympathetic Architecture (Nova Feature).
//!
//! # The Spark
//! The colony's structures take a beating from use, but what if they also "felt" the pain
//! of the Pops living inside them?
//!
//! # The Feature
//! Pops that die violently or experience an extreme mental breakdown near a Building cause
//! a small amount of structural damage to that Building. The base literally crumbles
//! under the emotional and physical weight of its inhabitants.

use crate::layer1::building::Building;
use crate::layer1::health::{Dead, Health};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

const TRAUMA_DAMAGE: f32 = 5.0;
const TRAUMA_RADIUS: u32 = 2;

/// System that damages nearby buildings when a pop dies.
pub fn sympathetic_architecture_system(
    pops: Query<&GridPosition, (With<Pop>, Added<Dead>)>,
    mut buildings: Query<(&GridPosition, &mut Health), With<Building>>,
) {
    for pop_pos in pops.iter() {
        for (building_pos, mut health) in buildings.iter_mut() {
            if pop_pos.distance_chebyshev(*building_pos) <= TRAUMA_RADIUS {
                health.take_damage(TRAUMA_DAMAGE);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(sympathetic_architecture_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_sympathetic_architecture_damages_buildings() {
        let mut world = World::new();

        // Target building in range
        let building = world
            .spawn((
                Building {
                    building_type: crate::layer1::building::BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Target building out of range
        let far_building = world
            .spawn((
                Building {
                    building_type: crate::layer1::building::BuildingType::Housing,
                },
                GridPosition { x: 15, y: 15 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Pop dies
        world.spawn((
            Pop,
            GridPosition { x: 6, y: 5 }, // Distance 1 (<= 2)
            Dead,                        // Will be picked up by Added<Dead>
        ));

        // First we need to make sure the trigger hits `Added` so let's run a tick to trigger changes
        world
            .run_system_once(sympathetic_architecture_system)
            .unwrap();

        let b_health = world.get::<Health>(building).unwrap();
        let fb_health = world.get::<Health>(far_building).unwrap();

        assert!(
            b_health.current < 100.0,
            "Building should take trauma damage"
        );
        // Strict clippy settings prohibit direct float comparisons
        assert!(
            (fb_health.current - 100.0).abs() < f32::EPSILON,
            "Far building should not be damaged"
        );
    }
}
