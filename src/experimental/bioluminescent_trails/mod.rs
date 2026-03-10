//! Bioluminescent Trails (Nova Feature).
//!
//! # The Spark
//! We have a `Morale` system and a `LightMap` system. What if extreme happiness
//! was infectious not just emotionally, but visually?
//!
//! # The Feature
//! Pops with extremely high morale (> 0.9) leave behind fading `BioluminescentTrail`
//! entities as they walk. These trails emit a soft, colorful light (`LightSource`).
//! This physically brightens the colony, pushing back the darkness penalty,
//! and creates beautiful, glowing desire paths that map the colony's joy.

use crate::layer1::lighting::LightSource;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

/// A fading trail of light left by a happy Pop.
#[derive(Component)]
pub struct BioluminescentTrail {
    /// Ticks remaining until the trail despawns.
    pub lifetime: u32,
    /// Maximum lifetime, used to calculate fade.
    pub max_lifetime: u32,
}

const TRAIL_LIFETIME: u32 = 100;
const MORALE_THRESHOLD: f32 = 0.9;
const MAX_TRAILS_PER_TILE: usize = 1;

/// System that checks for extremely happy Pops and spawns glowing footprints.
pub fn spawn_bioluminescent_trails_system(
    mut commands: Commands,
    pops: Query<(&GridPosition, &Morale), With<Pop>>,
    existing_trails: Query<&GridPosition, With<BioluminescentTrail>>,
) {
    let mut current_trails = std::collections::HashMap::new();
    for pos in existing_trails.iter() {
        *current_trails.entry(*pos).or_insert(0) += 1;
    }

    let mut to_spawn = Vec::new();

    for (pos, morale) in pops.iter() {
        if morale.value >= MORALE_THRESHOLD {
            let trail_count = current_trails.get(pos).copied().unwrap_or(0);
            if trail_count < MAX_TRAILS_PER_TILE {
                to_spawn.push(*pos);
                // Pre-emptively increment so we don't spawn multiple this frame for the same tile
                *current_trails.entry(*pos).or_insert(0) += 1;
            }
        }
    }

    for pos in to_spawn {
        commands.spawn((
            BioluminescentTrail {
                lifetime: TRAIL_LIFETIME,
                max_lifetime: TRAIL_LIFETIME,
            },
            LightSource { is_outdoor: false,
                radius: 2.0, // Soft, local glow
                intensity: 0.5,
                color: (50, 255, 100), // Bioluminescent green/blue
            },
            pos,
        ));
    }
}

/// System that fades and despawns bioluminescent trails.
pub fn fade_bioluminescent_trails_system(
    mut commands: Commands,
    mut trails: Query<(Entity, &mut BioluminescentTrail, &mut LightSource)>,
) {
    for (entity, mut trail, mut light) in trails.iter_mut() {
        if trail.lifetime > 0 {
            trail.lifetime -= 1;

            // Fade intensity based on remaining lifetime
            #[allow(clippy::cast_precision_loss)]
            let ratio = trail.lifetime as f32 / trail.max_lifetime as f32;
            light.intensity = 0.5 * ratio;
        } else {
            commands.entity(entity).despawn();
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        spawn_bioluminescent_trails_system,
        fade_bioluminescent_trails_system,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_spawn_trail_high_morale() {
        let mut world = World::new();

        // Happy Pop
        world.spawn((
            Pop,
            Morale {
                value: 0.95,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        world
            .run_system_once(spawn_bioluminescent_trails_system)
            .unwrap();

        // Check if trail was spawned
        let trails: Vec<_> = world.query::<&BioluminescentTrail>().iter(&world).collect();
        assert_eq!(trails.len(), 1, "One trail should be spawned");
    }

    #[test]
    fn test_no_spawn_low_morale() {
        let mut world = World::new();

        // Sad Pop
        world.spawn((
            Pop,
            Morale {
                value: 0.5,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        world
            .run_system_once(spawn_bioluminescent_trails_system)
            .unwrap();

        let trails: Vec<_> = world.query::<&BioluminescentTrail>().iter(&world).collect();
        assert_eq!(trails.len(), 0, "No trail should be spawned for low morale");
    }

    #[test]
    fn test_trail_fades_and_despawns() {
        let mut world = World::new();

        let entity = world
            .spawn((
                BioluminescentTrail {
                    lifetime: 1,
                    max_lifetime: 100,
                },
                LightSource { is_outdoor: false,
                    radius: 2.0,
                    intensity: 0.5,
                    color: (50, 255, 100),
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run fade system (lifetime 1 -> 0)
        world
            .run_system_once(fade_bioluminescent_trails_system)
            .unwrap();

        // It shouldn't despawn yet, just hits 0
        let light = world.get::<LightSource>(entity).unwrap();
        assert!(light.intensity < 0.1, "Intensity should fade to near 0");

        // Run again (lifetime 0 -> despawn)
        world
            .run_system_once(fade_bioluminescent_trails_system)
            .unwrap();

        assert!(
            world.get_entity(entity).is_err(),
            "Entity should be despawned"
        );
    }

    #[test]
    fn test_max_trails_per_tile() {
        let mut world = World::new();

        // Two Happy Pops on the same tile
        world.spawn((
            Pop,
            Morale {
                value: 0.95,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        world.spawn((
            Pop,
            Morale {
                value: 0.95,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        world
            .run_system_once(spawn_bioluminescent_trails_system)
            .unwrap();

        // Only one trail should spawn to prevent entity spam
        let trails: Vec<_> = world.query::<&BioluminescentTrail>().iter(&world).collect();
        assert_eq!(trails.len(), 1, "Only one trail should spawn per tile");
    }
}
