//! The Biosphere Empathy Link (Nova Feature).
//!
//! # The Spark
//! We have a `TerrainGrid` and `Morale`. What if the planet acts as an emotional conductor?
//!
//! # The Feature
//! Pops with the `Compassionate` trait standing on living terrain (`Grass`, `Tree`, `Shrub`, `Sapling`)
//! form a global empathic network. Their morale slowly equalizes towards the average morale
//! of all other Compassionate pops on living terrain. It turns the natural terrain into a giant emotional heatsink.

use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const EMPATHY_PULL_RATE: f32 = 0.05;

/// Checks if a terrain type is considered part of the living biosphere.
fn is_living_terrain(terrain: TerrainType) -> bool {
    matches!(
        terrain,
        TerrainType::Grass | TerrainType::Tree | TerrainType::Shrub | TerrainType::Sapling
    )
}

/// System that connects Compassionate pops on living terrain and averages their morale.
pub fn biosphere_empathy_system(
    mut pops: Query<(&GridPosition, &mut Morale, &Traits), With<Pop>>,
    terrain: Res<TerrainGrid>,
) {
    if pops.is_empty() {
        return;
    }

    // Pass 1: Collect network data
    let mut total_morale = 0.0;
    let mut network_count = 0;
    let mut network_entities = Vec::new();

    // Iterate once to find all pops in the network
    for (pos, morale, traits) in pops.iter() {
        if traits.0.contains(&Trait::Compassionate) {
            if pos.x >= 0
                && (pos.x as usize) < terrain.width
                && pos.y >= 0
                && (pos.y as usize) < terrain.height
            {
                if let Some(tile) = terrain.get(pos.x as usize, pos.y as usize) {
                    if is_living_terrain(tile) {
                        total_morale += morale.value;
                        network_count += 1;
                        network_entities.push(*pos);
                    }
                }
            }
        }
    }

    if network_count <= 1 {
        return; // Need at least 2 pops to equalize
    }

    let avg_morale = total_morale / network_count as f32;

    // Pass 2: Apply empathic pull to members of the network
    for (pos, mut morale, traits) in pops.iter_mut() {
        if traits.0.contains(&Trait::Compassionate) {
            // Re-check terrain or just use the collected positions
            if network_entities.contains(pos) {
                // Pull morale towards average
                let delta = avg_morale - morale.value;
                morale.value = (morale.value + (delta * EMPATHY_PULL_RATE)).clamp(0.0, 1.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(biosphere_empathy_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;
    use std::collections::HashSet;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut terrain = crate::layer1::terrain::generate_terrain(10, 10);

        // Setup known terrain
        terrain.set(0, 0, TerrainType::Grass);
        terrain.set(1, 1, TerrainType::Grass);
        terrain.set(2, 2, TerrainType::Rock);
        terrain.set(3, 3, TerrainType::Grass);

        world.insert_resource(terrain);
        world
    }

    #[test]
    fn test_biosphere_empathy_equalizes_morale() {
        let mut world = setup_world();

        let traits = Traits(HashSet::from([Trait::Compassionate]));

        let p1 = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 }, // Grass
                Morale {
                    value: 0.1,
                    ..Default::default()
                },
                traits.clone(),
            ))
            .id();

        let p2 = world
            .spawn((
                Pop,
                GridPosition { x: 1, y: 1 }, // Grass
                Morale {
                    value: 0.9,
                    ..Default::default()
                },
                traits,
            ))
            .id();

        world.run_system_once(biosphere_empathy_system).unwrap();

        let m1 = world.get::<Morale>(p1).unwrap().value;
        let m2 = world.get::<Morale>(p2).unwrap().value;

        // Average is 0.5.
        // p1 moves from 0.1 towards 0.5 (+0.02) = 0.12
        // p2 moves from 0.9 towards 0.5 (-0.02) = 0.88
        assert!(m1 > 0.1, "Low morale should be pulled up");
        assert!(m2 < 0.9, "High morale should be pulled down");
    }

    #[test]
    fn test_biosphere_empathy_ignores_non_compassionate() {
        let mut world = setup_world();

        let traits_compassionate = Traits(HashSet::from([Trait::Compassionate]));
        let traits_normal = Traits(HashSet::new());

        let p1 = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 }, // Grass
                Morale {
                    value: 0.1,
                    ..Default::default()
                },
                traits_compassionate,
            ))
            .id();

        let p2 = world
            .spawn((
                Pop,
                GridPosition { x: 1, y: 1 }, // Grass
                Morale {
                    value: 0.9,
                    ..Default::default()
                },
                traits_normal,
            ))
            .id();

        world.run_system_once(biosphere_empathy_system).unwrap();

        let m1 = world.get::<Morale>(p1).unwrap().value;
        let m2 = world.get::<Morale>(p2).unwrap().value;

        // Since p2 is not compassionate, p1 is alone in network, no equalization.
        assert!(
            (m1 - 0.1).abs() < f32::EPSILON,
            "Should not equalize if alone in network"
        );
        assert!(
            (m2 - 0.9).abs() < f32::EPSILON,
            "Non-compassionate should not be affected"
        );
    }

    #[test]
    fn test_biosphere_empathy_ignores_rock_terrain() {
        let mut world = setup_world();

        let traits = Traits(HashSet::from([Trait::Compassionate]));

        let p1 = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 }, // Grass
                Morale {
                    value: 0.1,
                    ..Default::default()
                },
                traits.clone(),
            ))
            .id();

        let p2 = world
            .spawn((
                Pop,
                GridPosition { x: 2, y: 2 }, // Rock
                Morale {
                    value: 0.9,
                    ..Default::default()
                },
                traits,
            ))
            .id();

        world.run_system_once(biosphere_empathy_system).unwrap();

        let m1 = world.get::<Morale>(p1).unwrap().value;
        let m2 = world.get::<Morale>(p2).unwrap().value;

        // p2 is on rock, not in network.
        assert!((m1 - 0.1).abs() < f32::EPSILON);
        assert!((m2 - 0.9).abs() < f32::EPSILON);
    }
}
