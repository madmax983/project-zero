//! The Echoing Object (Nova Feature).
//!
//! # The Spark
//! We have `Spirit` mechanics in `animism.rs` where items gain traits based on their use.
//! We also have an `Acoustic` system (`NoiseSource`) that usually applies to buildings or combat.
//! What if a highly `Haunted` item begins to spontaneously emit auditory hallucinations?
//!
//! # The Feature
//! The `echoing_object_system` checks `Item` or `Equipment` entities that possess a
//! `Spirit` component containing the `SpiritTrait::Haunted` trait. If their spirit level
//! is high enough, there is a chance they will spontaneously spawn a `NoiseSource` entity
//! at their current location (representing the voices of past owners). This creates local
//! stress and unease for nearby Pops.
//!
//! This connects the history of an object (Animism) to the physical simulation (Acoustics)
//! and psychology (Morale).

use crate::layer1::culture::animism::{Spirit, SpiritTrait};
use crate::layer1::map::GridPosition;
use crate::layer1::physics::acoustic::NoiseSource;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component indicating an active echoing anomaly.
#[derive(Component)]
pub struct EchoingAnomaly {
    /// Ticks remaining until the anomaly dissipates.
    pub duration: u32,
}

const HAUNTED_THRESHOLD_LEVEL: u32 = 5;
const ECHO_CHANCE_PER_TICK: f64 = 0.005;

/// System that causes Haunted objects to spontaneously emit NoiseSources.
pub fn echoing_object_system(
    mut commands: Commands,
    query: Query<(&GridPosition, &Spirit)>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    let mut rng = rand::thread_rng();
    let mut echoed = false;

    for (pos, spirit) in query.iter() {
        if spirit.level >= HAUNTED_THRESHOLD_LEVEL
            && spirit.traits.contains(&SpiritTrait::Haunted)
            && rng.gen_bool(ECHO_CHANCE_PER_TICK)
        {
            // Spawn a temporary noise source at the object's location
            commands.spawn((
                EchoingAnomaly { duration: 50 }, // Lasts 50 ticks
                *pos,
                NoiseSource {
                    radius: 8.0,
                    intensity: 0.8, // Disturbing but not deafening
                },
            ));
            echoed = true;
        }
    }

    if echoed {
        if let Some(ref mut log_res) = log {
            log_res.add_colored(
                "A distant, unsettling whisper emanates from an old tool...".to_string(),
                ratatui::style::Color::DarkGray,
            );
        }
    }
}

/// System to despawn Echoing Anomalies when their duration expires.
pub fn echoing_anomaly_decay_system(
    mut commands: Commands,
    mut anomalies: Query<(Entity, &mut EchoingAnomaly)>,
) {
    for (entity, mut anomaly) in anomalies.iter_mut() {
        if anomaly.duration == 0 {
            commands.entity(entity).despawn();
        } else {
            anomaly.duration -= 1;
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((echoing_object_system, echoing_anomaly_decay_system));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_echoing_object_spawns_noise_source() {
        let mut world = World::new();

        // Spawn a haunted object
        let haunted_spirit = Spirit {
            experience: 1000,
            level: HAUNTED_THRESHOLD_LEVEL,
            traits: vec![SpiritTrait::Haunted],
        };

        world.spawn((GridPosition { x: 10, y: 10 }, haunted_spirit));

        // We run the system 1000 times to almost guarantee a trigger since probability is 0.005 per tick
        for _ in 0..1000 {
            world.run_system_once(echoing_object_system).unwrap();
        }

        // Check if an anomaly and noise source were spawned
        let mut query = world.query::<(&EchoingAnomaly, &NoiseSource)>();
        let count = query.iter(&world).count();
        assert!(count > 0, "At least one echoing anomaly should have spawned");
    }

    #[test]
    fn test_echoing_anomaly_decay() {
        let mut world = World::new();

        let anomaly = world.spawn(EchoingAnomaly { duration: 1 }).id();

        // 1st tick: reduces duration to 0
        world.run_system_once(echoing_anomaly_decay_system).unwrap();

        // 2nd tick: despawns
        world.run_system_once(echoing_anomaly_decay_system).unwrap();

        assert!(
            world.get_entity(anomaly).is_err(),
            "Anomaly should be despawned after duration reaches 0"
        );
    }
}
