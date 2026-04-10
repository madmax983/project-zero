//! Bio-Acoustic Miasma System.
//!
//! (Implementation of Spec 570)
//!
//! This module introduces `MiasmaCloud` entities which cause `Stress` and
//! generate `Paranoia` in Pops. Miasma clouds record "Secrets" from high-stress
//! Pops and broadcast them to other Pops later.
//!
//! # Examples
//!
//! ```
//! use bevy_ecs::prelude::*;
//! use scale::layer1::bio_acoustic_miasma::{MiasmaCloud, ParanoiaTracker, MiasmaRecordedSecret, record_miasma_secret, broadcast_miasma_secrets};
//! use scale::layer1::map::GridPosition;
//! use scale::layer1::pop::Pop;
//! use scale::layer1::stress::StressTracker;
//! use scale::layer1::chronicle::{Chronicle, AddChronicleEvent};
//!
//! let mut world = World::new();
//! world.init_resource::<MiasmaRecordedSecret>();
//! world.init_resource::<Chronicle>();
//! world.init_resource::<Events<AddChronicleEvent>>();
//!
//! let pos = GridPosition { x: 5, y: 5 };
//!
//! // Spawn a cloud
//! world.spawn(MiasmaCloud { position: pos, lifetime: 10, intensity: 1.0 });
//!
//! // Spawn a highly stressed Pop
//! world.spawn((Pop, pos, StressTracker { accumulated_stress: 90.0 }));
//!
//! // Run record system
//! let mut schedule = Schedule::default();
//! schedule.add_systems(record_miasma_secret);
//! schedule.run(&mut world);
//!
//! // The secret should be recorded
//! let secrets = world.get_resource::<MiasmaRecordedSecret>().unwrap();
//! assert_eq!(secrets.0.len(), 1);
//!
//! // Spawn a listener
//! let listener = world.spawn((Pop, pos, ParanoiaTracker { level: 0 })).id();
//!
//! // Run broadcast system
//! let mut schedule2 = Schedule::default();
//! schedule2.add_systems(broadcast_miasma_secrets);
//! schedule2.run(&mut world);
//!
//! let paranoia = world.get::<ParanoiaTracker>(listener).unwrap().level;
//! assert!(paranoia > 0);
//!
//! let secrets_after = world.get_resource::<MiasmaRecordedSecret>().unwrap();
//! assert!(secrets_after.0.is_empty());
//! ```

use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;
use rand::seq::IteratorRandom;

#[derive(Component)]
pub struct MiasmaCloud {
    pub position: GridPosition,
    pub lifetime: u32,
    pub intensity: f32,
}

#[derive(Resource, Default)]
pub struct MiasmaRecordedSecret(pub Vec<String>);

#[derive(Component)]
pub struct ParanoiaTracker {
    pub level: u32,
}

pub fn update_miasma_clouds(mut commands: Commands, mut query: Query<(Entity, &mut MiasmaCloud)>) {
    for (entity, mut cloud) in query.iter_mut() {
        if cloud.lifetime > 0 {
            cloud.lifetime -= 1;
        }
        if cloud.lifetime == 0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn apply_miasma_stress(
    cloud_query: Query<&MiasmaCloud>,
    mut pop_query: Query<(&GridPosition, &mut StressTracker), With<Pop>>,
) {
    for cloud in cloud_query.iter() {
        for (pop_pos, mut stress) in pop_query.iter_mut() {
            if cloud.position == *pop_pos {
                // Apply stress based on intensity
                stress.accumulated_stress += cloud.intensity;
            }
        }
    }
}

pub fn record_miasma_secret(
    mut secrets: ResMut<MiasmaRecordedSecret>,
    cloud_query: Query<&MiasmaCloud>,
    pop_query: Query<(&GridPosition, &StressTracker), With<Pop>>,
) {
    // For each pop with high stress, check if they are in a miasma cloud
    for (pop_pos, stress) in pop_query.iter() {
        if stress.accumulated_stress > 80.0 {
            for cloud in cloud_query.iter() {
                if cloud.position == *pop_pos {
                    // Record a generic secret for now
                    secrets.0.push("high_stress_complaint".to_string());
                }
            }
        }
    }
}

pub fn broadcast_miasma_secrets(
    mut secrets: ResMut<MiasmaRecordedSecret>,
    mut paranoia_query: Query<&mut ParanoiaTracker>,
    mut events: EventWriter<AddChronicleEvent>,
) {
    if !secrets.0.is_empty() {
        for mut paranoia in paranoia_query.iter_mut() {
            // Increase paranoia
            paranoia.level += 1;

            // Optional: emit chronicle event for the broadcast
            let mut rng = rand::thread_rng();
            if let Some(secret) = secrets.0.iter().choose(&mut rng) {
                events.send(AddChronicleEvent {
                    text: format!("A miasma cloud whispers a secret: {secret}"),
                    importance: EventImportance::Minor,
                });
            }
        }
        // Consume secrets after broadcasting
        secrets.0.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::Chronicle;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_miasma_cloud_despawns_when_lifetime_ends() {
        let mut world = World::new();

        let entity = world
            .spawn(MiasmaCloud {
                position: GridPosition { x: 0, y: 0 },
                lifetime: 1,
                intensity: 1.0,
            })
            .id();

        world.run_system_once(update_miasma_clouds).unwrap();

        assert!(
            world.get_entity(entity).is_err(),
            "Cloud should be despawned"
        );
    }

    #[test]
    fn test_miasma_applies_stress() {
        let mut world = World::new();
        let pos = GridPosition { x: 5, y: 5 };

        world.spawn(MiasmaCloud {
            position: pos,
            lifetime: 5,
            intensity: 10.0,
        });

        let pop = world
            .spawn((
                Pop,
                pos,
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(apply_miasma_stress).unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert_eq!(stress.accumulated_stress, 10.0);
    }

    #[test]
    fn test_miasma_records_interaction() {
        let mut world = World::new();
        world.init_resource::<MiasmaRecordedSecret>();

        let pos = GridPosition { x: 5, y: 5 };
        world.spawn(MiasmaCloud {
            position: pos,
            lifetime: 5,
            intensity: 1.0,
        });
        world.spawn((
            Pop,
            pos,
            StressTracker {
                accumulated_stress: 90.0,
            },
        ));

        world.run_system_once(record_miasma_secret).unwrap();

        let secrets = world.get_resource::<MiasmaRecordedSecret>().unwrap();
        assert!(secrets.0.contains(&"high_stress_complaint".to_string()));
    }

    #[test]
    fn test_miasma_broadcasts_secret() {
        let mut world = World::new();
        world.insert_resource(MiasmaRecordedSecret(vec!["plot_strike".to_string()]));
        world.init_resource::<Chronicle>();
        world.init_resource::<Events<AddChronicleEvent>>();

        let listener = world
            .spawn((ParanoiaTracker { level: 0 }, GridPosition { x: 1, y: 1 }))
            .id();

        world.run_system_once(broadcast_miasma_secrets).unwrap();

        let paranoia = world.get::<ParanoiaTracker>(listener).unwrap();
        assert_eq!(paranoia.level, 1);

        let secrets = world.get_resource::<MiasmaRecordedSecret>().unwrap();
        assert!(secrets.0.is_empty());
    }
}
