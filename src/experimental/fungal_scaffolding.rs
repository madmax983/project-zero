//! Fungal Scaffolding (Nova Feature).
//!
//! # The Spark
//! We have `Structure` (HP for buildings), `ClutterGrid`, and `SporeNetwork`.
//!
//! # The Feature
//! When a Building is damaged (`Structure.current_hp` < `Structure.max_hp`), and there is an active `SporeNetwork`, the fungal network organically patches the building. It naturally regenerates a small amount of the building's HP per tick, scaling with the network's strength. But as a side effect, it spawns `Clutter` (mycelial overgrowth) around the building.
//!
//! # The Potential
//! Connects the spore mechanics to base maintenance. Players using the Spore Network get free auto-repair on their walls and buildings, but have to deal with the resulting mess (Clutter) it leaves behind, creating a symbiotic loop between base repair and cleanliness.

use crate::layer1::architecture::building::Building;
use crate::layer1::architecture::structure::Structure;
use crate::layer1::clutter::ClutterGrid;
use crate::layer1::fungal_network::SporeNetwork;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

const REPAIR_AMOUNT_PER_TAP: f32 = 0.05;
const CLUTTER_SPAWN_AMOUNT: f32 = 0.5;

/// System that heals buildings via fungal scaffolding.
pub fn fungal_scaffolding_system(
    network: Option<Res<SporeNetwork>>,
    clutter_grid: Option<ResMut<ClutterGrid>>,
    mut buildings: Query<(&GridPosition, &mut Structure), With<Building>>,
) {
    let Some(network) = network else { return };
    if network.active_taps == 0 { return };

    let Some(mut clutter) = clutter_grid else { return };

    let repair_amount = REPAIR_AMOUNT_PER_TAP * (network.active_taps as f32);

    for (pos, mut structure) in &mut buildings {
        if structure.current_hp < structure.max_hp {
            // Apply repair
            structure.current_hp = (structure.current_hp + repair_amount).min(structure.max_hp);

            // Add clutter at the building's position
            if pos.x >= 0 && pos.y >= 0 {
                let x = pos.x as usize;
                let y = pos.y as usize;

                // Directly replicate add_clutter logic to avoid borrowing issues or reliance on missing method if not public
                let current_clutter = clutter.get(x, y);
                clutter.set(x, y, (current_clutter + CLUTTER_SPAWN_AMOUNT).min(100.0));
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(fungal_scaffolding_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::architecture::building::BuildingType;

    #[test]
    fn test_fungal_scaffolding_repairs_and_clutters() {
        let mut world = World::new();

        // Setup SporeNetwork
        world.insert_resource(SporeNetwork { active_taps: 2 });

        // Setup ClutterGrid
        world.insert_resource(ClutterGrid::new(10, 10));

        // Spawn a damaged building
        let building_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        world.run_system_once(fungal_scaffolding_system).unwrap();

        let structure = world.get::<Structure>(building_entity).unwrap();

        // Repair should be 2 taps * 0.05 = 0.1
        assert!(
            (structure.current_hp - 50.1).abs() < f32::EPSILON,
            "Building should be repaired by 0.1, got {}",
            structure.current_hp
        );

        let clutter_grid = world.resource::<ClutterGrid>();
        let clutter_amount = clutter_grid.get(5, 5);

        assert!(
            (clutter_amount - CLUTTER_SPAWN_AMOUNT).abs() < f32::EPSILON,
            "Clutter should be spawned at the building location, got {}",
            clutter_amount
        );
    }

    #[test]
    fn test_fungal_scaffolding_no_active_network() {
        let mut world = World::new();

        world.insert_resource(SporeNetwork { active_taps: 0 });
        world.insert_resource(ClutterGrid::new(10, 10));

        let building_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        world.run_system_once(fungal_scaffolding_system).unwrap();

        let structure = world.get::<Structure>(building_entity).unwrap();

        assert_eq!(
            structure.current_hp, 50.0,
            "Building should not be repaired if no active taps"
        );

        let clutter_grid = world.resource::<ClutterGrid>();
        assert_eq!(clutter_grid.get(5, 5), 0.0);
    }
}
