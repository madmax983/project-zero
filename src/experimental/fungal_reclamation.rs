//! Fungal Reclamation (Nova Feature).
//!
//! # The Spark
//! We have a `SporeNetwork` from `fungal_network.rs` and `BuildingRemovedEvent`.
//!
//! # The Feature
//! When a building is removed/destroyed, if the `SporeNetwork` has active taps (`active_taps > 0`),
//! the mycelial network immediately reclaims the ruins. It rapidly generates `Clutter` on that tile
//! representing a sudden spore bloom, and provides a small flat boost to the global `PopCollectivism` level.
//!
//! # The Potential
//! Connects building destruction (which is normally bad) to the fungal mechanics (which is weird).
//! If players want to rapidly advance their `PopCollectivism` (perhaps to unlock specific mechanics),
//! they might intentionally construct and demolish buildings to feed the `SporeNetwork`.

use crate::layer1::clutter::ClutterGrid;
use crate::layer1::core::events::BuildingRemovedEvent;
use crate::layer1::fungal_network::{PopCollectivism, SporeNetwork};
use bevy_ecs::prelude::*;

/// Fungal Reclamation system.
///
/// Triggers when buildings are destroyed. If the `SporeNetwork` is active, it reclaims the tile
/// by spawning a large amount of clutter (spore bloom) and granting a sudden boost to collectivism.
pub fn fungal_reclamation_system(
    mut events: EventReader<BuildingRemovedEvent>,
    network: Option<Res<SporeNetwork>>,
    mut clutter_grid: Option<ResMut<ClutterGrid>>,
    mut pop_query: Query<&mut PopCollectivism>,
) {
    if let Some(net) = network {
        if net.active_taps > 0 {
            for event in events.read() {
                // Generate massive clutter (spore bloom) at the ruined building location
                if let Some(ref mut grid) = clutter_grid {
                    // Make sure the coordinates are non-negative to avoid panics on cast
                    if event.position.x >= 0 && event.position.y >= 0 {
                        let x = event.position.x as usize;
                        let y = event.position.y as usize;

                        if x < grid.width && y < grid.height {
                            grid.add_clutter(x, y, 50.0);
                        }
                    }
                }

                // Boost collectivism for all pops to represent the psychic connection to the mycelium
                for mut pop in pop_query.iter_mut() {
                    pop.level += 0.05; // Significant boost per building destroyed
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(fungal_reclamation_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::BuildingType;
    use crate::layer1::core::map::GridPosition;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Events::<BuildingRemovedEvent>::default());
        world.insert_resource(ClutterGrid::new(10, 10));
        world
    }

    #[test]
    fn test_fungal_reclamation_with_active_taps() {
        let mut world = setup_world();
        world.insert_resource(SporeNetwork { active_taps: 1 });

        let pop_entity = world.spawn(PopCollectivism { level: 0.1 }).id();

        world
            .resource_mut::<Events<BuildingRemovedEvent>>()
            .send(BuildingRemovedEvent {
                entity: Entity::from_raw(1),
                position: GridPosition { x: 5, y: 5 },
                building_type: BuildingType::Housing,
            });

        world.run_system_once(fungal_reclamation_system).unwrap();

        // Verify Clutter added
        let grid = world.resource::<ClutterGrid>();
        assert!(
            grid.get(5, 5) >= 50.0,
            "Clutter should be added upon reclamation"
        );

        // Verify Collectivism boosted
        let collectivism = world.get::<PopCollectivism>(pop_entity).unwrap();
        assert!(collectivism.level > 0.14, "Collectivism should increase");
    }

    #[test]
    fn test_fungal_reclamation_no_active_taps() {
        let mut world = setup_world();
        world.insert_resource(SporeNetwork { active_taps: 0 }); // No active taps

        let pop_entity = world.spawn(PopCollectivism { level: 0.1 }).id();

        world
            .resource_mut::<Events<BuildingRemovedEvent>>()
            .send(BuildingRemovedEvent {
                entity: Entity::from_raw(1),
                position: GridPosition { x: 5, y: 5 },
                building_type: BuildingType::Housing,
            });

        world.run_system_once(fungal_reclamation_system).unwrap();

        // Verify Clutter NOT added
        let grid = world.resource::<ClutterGrid>();
        assert_eq!(
            grid.get(5, 5),
            0.0,
            "Clutter should not be added if no active taps"
        );

        // Verify Collectivism NOT boosted
        let collectivism = world.get::<PopCollectivism>(pop_entity).unwrap();
        assert!(
            (collectivism.level - 0.1).abs() < f32::EPSILON,
            "Collectivism should not increase"
        );
    }
}
