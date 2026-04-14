#![allow(clippy::type_complexity)]
//! The Feral Choir (Nova Feature).
//!
//! # The Spark
//! We have Hostile Fauna and Acoustic Simulation. What if animals can sing together to debuff the colony?
//!
//! # The Feature
//! When multiple `Fauna` entities are within range of each other, they form a "Feral Choir".
//! Their combined howling creates a massive `NoiseSource` that applies a global morale penalty to all Pops who can hear it.

use crate::layer1::fauna::Fauna;
use crate::layer1::map::GridPosition;
use crate::layer1::physics::acoustic::NoiseSource;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Component indicating an entity is the source of a Feral Choir's noise.
#[derive(Component)]
pub struct FeralChoir;

/// Component indicating a fauna entity is participating in a Feral Choir.
#[derive(Component)]
pub struct ChoirMember;

const CHOIR_CLUSTER_RADIUS: u32 = 5;
const MIN_CHOIR_SIZE: usize = 3;

/// System to detect and form a Feral Choir from clustered Fauna.
pub fn detect_feral_choir_system(
    mut commands: Commands,
    query: Query<(Entity, &GridPosition), (With<Fauna>, Without<ChoirMember>)>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    let fauna_list: Vec<(Entity, GridPosition)> = query.iter().map(|(e, p)| (e, *p)).collect();

    // Iterate through available fauna to find a cluster
    let mut processed = HashSet::new();

    for &(entity1, pos1) in &fauna_list {
        if processed.contains(&entity1) {
            continue;
        }

        let mut cluster = vec![entity1];

        for &(entity2, pos2) in &fauna_list {
            if entity1 == entity2 || processed.contains(&entity2) {
                continue;
            }

            if pos1.distance_chebyshev(pos2) <= CHOIR_CLUSTER_RADIUS {
                cluster.push(entity2);
            }
        }

        if cluster.len() >= MIN_CHOIR_SIZE {
            // Form a choir!
            for &e in &cluster {
                commands.entity(e).insert(ChoirMember);
                processed.insert(e);
            }

            // Spawn the FeralChoir entity at the center (the first entity's position)
            commands.spawn((
                FeralChoir,
                pos1,
                NoiseSource {
                    radius: 20.0,
                    intensity: 1.0,
                },
            ));

            if let Some(ref mut log_res) = log {
                log_res.add_colored(
                    "DANGER: A Feral Choir has begun howling in the distance!".to_string(),
                    ratatui::style::Color::Red,
                );
            }
        }
    }
}

/// System to dissolve a Feral Choir if its members disperse or die.
pub fn dissolve_feral_choir_system(
    mut commands: Commands,
    choirs: Query<(Entity, &GridPosition), With<FeralChoir>>,
    fauna: Query<(Entity, &GridPosition), With<ChoirMember>>,
) {
    for (choir_entity, choir_pos) in choirs.iter() {
        let mut members_in_range = Vec::new();

        for (fauna_entity, fauna_pos) in fauna.iter() {
            if choir_pos.distance_chebyshev(*fauna_pos) <= CHOIR_CLUSTER_RADIUS {
                members_in_range.push(fauna_entity);
            }
        }

        if members_in_range.len() < MIN_CHOIR_SIZE {
            // Dissolve the choir
            commands.entity(choir_entity).despawn();

            // Remove ChoirMember from those that were in range (others might have already wandered off or died)
            for &e in &members_in_range {
                commands.entity(e).remove::<ChoirMember>();
            }

            // Note: If a ChoirMember wandered far away from the choir center, it might still have the component.
            // We should ideally track which choir they belong to, but for simplicity, any ChoirMember far from
            // all choirs will eventually lose it or we can just clean them up globally. Let's do a global cleanup.
            for (fauna_entity, fauna_pos) in fauna.iter() {
                // If this fauna is not within range of ANY active choir, remove the component.
                let mut near_any = false;
                for (_, other_choir_pos) in choirs.iter() {
                    // Skip the one we are dissolving
                    if choir_pos == other_choir_pos {
                        continue;
                    }
                    if other_choir_pos.distance_chebyshev(*fauna_pos) <= CHOIR_CLUSTER_RADIUS {
                        near_any = true;
                        break;
                    }
                }

                if !near_any {
                    commands.entity(fauna_entity).remove::<ChoirMember>();
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((detect_feral_choir_system, dissolve_feral_choir_system));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_choir_formation_and_dissolution() {
        let mut world = World::new();

        // Spawn 3 Fauna entities close to each other
        let f1 = world
            .spawn((Fauna::default(), GridPosition { x: 0, y: 0 }))
            .id();
        let f2 = world
            .spawn((Fauna::default(), GridPosition { x: 1, y: 0 }))
            .id();
        let f3 = world
            .spawn((Fauna::default(), GridPosition { x: 2, y: 0 }))
            .id();

        // Detect
        world.run_system_once(detect_feral_choir_system).unwrap();

        assert!(world.get::<ChoirMember>(f1).is_some());
        assert!(world.get::<ChoirMember>(f2).is_some());
        assert!(world.get::<ChoirMember>(f3).is_some());

        assert_eq!(world.query::<&FeralChoir>().iter(&world).count(), 1);

        // Move f3 far away
        world.get_mut::<GridPosition>(f3).unwrap().x = 100;

        // Dissolve
        world.run_system_once(dissolve_feral_choir_system).unwrap();

        assert!(world.get::<ChoirMember>(f1).is_none());
        assert!(world.get::<ChoirMember>(f2).is_none());
        assert!(world.get::<ChoirMember>(f3).is_none());

        assert_eq!(world.query::<&FeralChoir>().iter(&world).count(), 0);
    }
}
