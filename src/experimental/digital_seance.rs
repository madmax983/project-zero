#![allow(clippy::type_complexity)]
//! Digital Seance (Nova Feature).
//!
//! # The Spark
//! We have a `HauntedGrid` tracking deaths and `ServerBank`s providing `DataStorage` and `Knowledge` capacity.
//! What if we connect the dead to the network?
//!
//! # The Feature
//! If a `BuildingType::ServerBank` is constructed on a tile with a positive death count in the `HauntedGrid`,
//! it gains the `NecroComputingNode` component. This allows the server to passively generate small amounts
//! of `Knowledge` every tick by analyzing the echoes of the dead.
//! However, tapping into the afterlife isn't silent: it causes the ServerBank to emit a massive `NoiseSource`
//! that behaves as a ghostly wailing, degrading the morale of nearby living Pops.

use crate::experimental::the_haunted_cartographer::HauntedGrid;
use crate::layer1::architecture::Building;
use crate::layer1::architecture::BuildingType;
use crate::layer1::map::GridPosition;
use crate::layer1::physics::acoustic::NoiseSource;
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

/// Component indicating a ServerBank has connected to the HauntedGrid and is performing necro-computing.
#[derive(Component)]
pub struct NecroComputingNode;

/// System to detect ServerBanks built on haunted tiles and turn them into NecroComputingNodes.
pub fn detect_necro_computing_system(
    mut commands: Commands,
    query: Query<(Entity, &Building, &GridPosition), Added<Building>>,
    haunted_grid: Res<HauntedGrid>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    for (entity, building, pos) in query.iter() {
        if building.building_type == BuildingType::ServerBank {
            if let Some(&death_count) = haunted_grid.death_counts.get(pos) {
                if death_count > 0 {
                    commands.entity(entity).insert((
                        NecroComputingNode,
                        NoiseSource {
                            radius: 10.0,
                            intensity: 0.8, // Loud, disturbing noise
                        },
                    ));

                    if let Some(ref mut log_res) = log {
                        log_res.add_colored(
                            "WARNING: A new ServerBank has synced with anomalous echoes of the dead. It is now a NecroComputing Node.".to_string(),
                            ratatui::style::Color::Red,
                        );
                    }
                }
            }
        }
    }
}

/// System to passively generate Knowledge from NecroComputingNodes.
pub fn necro_computing_generation_system(
    query: Query<(&NecroComputingNode, &GridPosition)>,
    haunted_grid: Res<HauntedGrid>,
    mut resources: ResMut<ColonyResources>,
) {
    for (_, pos) in query.iter() {
        if let Some(&death_count) = haunted_grid.death_counts.get(pos) {
            // Generates 0.1 Knowledge per death on the tile per tick.
            resources.add_knowledge(death_count as f32 * 0.1);
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        detect_necro_computing_system,
        necro_computing_generation_system,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_necro_computing_detection_and_generation() {
        let mut world = World::new();

        let mut haunted_grid = HauntedGrid::default();
        haunted_grid
            .death_counts
            .insert(GridPosition { x: 5, y: 5 }, 2);
        world.insert_resource(haunted_grid);
        world.insert_resource(ColonyResources::default());

        let server_bank = world
            .spawn((
                Building {
                    building_type: BuildingType::ServerBank,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world
            .run_system_once(detect_necro_computing_system)
            .unwrap();

        // Should have NecroComputingNode and NoiseSource
        assert!(world.get::<NecroComputingNode>(server_bank).is_some());
        assert!(world.get::<NoiseSource>(server_bank).is_some());

        // Run generation system
        world
            .run_system_once(necro_computing_generation_system)
            .unwrap();

        // Should generate 2 deaths * 0.1 = 0.2 Knowledge
        let resources = world.resource::<ColonyResources>();
        assert!((resources.knowledge - 0.2).abs() < f32::EPSILON);
    }
}
