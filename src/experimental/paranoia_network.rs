//! The Paranoia Network (Nova Feature).
//!
//! # The Spark
//! We have a `StressTracker` system and an `AffinityChange` system (social relationships).
//! What if intense stress didn't just cause personal breakdowns, but actively degraded
//! the social fabric of the colony?
//!
//! # The Feature
//! When a Pop's stress exceeds a critical threshold, they begin experiencing "Paranoia".
//! This manifests as a localized aura. Any other Pop that comes too close triggers
//! a negative `AffinityChange` event, meaning the paranoid Pop actively starts disliking
//! their neighbors. This creates a negative feedback loop where high-stress areas
//! become hotbeds of rivalry and social decay.

use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::social::AffinityChange;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

const PARANOIA_STRESS_THRESHOLD: f32 = 90.0;
const PARANOIA_RADIUS: u32 = 3;
const PARANOIA_AFFINITY_PENALTY: f32 = -0.5;

/// Component indicating a Pop is currently experiencing paranoia.
/// Used to prevent spamming events every single tick without a cooldown,
/// or to just visually mark them if needed. We'll use a simple cooldown here.
#[derive(Component)]
pub struct ParanoiaCooldown {
    pub ticks_remaining: u32,
}

const COOLDOWN_TICKS: u32 = 100;

/// System that causes highly stressed Pops to project their paranoia
/// onto nearby Pops, degrading their relationships.
pub fn paranoia_network_system(
    mut commands: Commands,
    mut pops: Query<
        (
            Entity,
            &GridPosition,
            &StressTracker,
            Option<&mut ParanoiaCooldown>,
        ),
        With<Pop>,
    >,
    mut affinity_events: EventWriter<AffinityChange>,
) {
    let mut paranoid_sources = Vec::new();

    // 1. Identify sources of paranoia
    for (entity, pos, stress, mut cooldown_opt) in pops.iter_mut() {
        if let Some(ref mut cooldown) = cooldown_opt {
            if cooldown.ticks_remaining > 0 {
                cooldown.ticks_remaining -= 1;
                continue;
            } else {
                commands.entity(entity).remove::<ParanoiaCooldown>();
            }
        } else if stress.accumulated_stress >= PARANOIA_STRESS_THRESHOLD {
            paranoid_sources.push((entity, *pos));
            commands.entity(entity).insert(ParanoiaCooldown {
                ticks_remaining: COOLDOWN_TICKS,
            });
        }
    }

    if paranoid_sources.is_empty() {
        return;
    }

    // 2. Find nearby targets and apply negative affinity
    // Note: O(N*M) but M (paranoid sources) is typically very small.
    for (target_entity, target_pos, _, _) in pops.iter() {
        for (source_entity, source_pos) in &paranoid_sources {
            if *source_entity == target_entity {
                continue;
            }

            if target_pos.distance_chebyshev(*source_pos) <= PARANOIA_RADIUS {
                // The paranoid pop dislikes the target pop for being too close
                affinity_events.send(AffinityChange {
                    source: *source_entity,
                    target: target_entity,
                    amount: PARANOIA_AFFINITY_PENALTY,
                });
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(paranoia_network_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Events::<AffinityChange>::default());
        world
    }

    #[test]
    fn test_paranoia_network_triggers_on_high_stress() {
        let mut world = setup_world();

        // High stress pop (Source)
        let source = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                StressTracker {
                    accumulated_stress: 95.0,
                },
            ))
            .id();

        // Target pop in range
        let target = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 5 },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(paranoia_network_system).unwrap();

        // Verify cooldown was added
        assert!(world.get::<ParanoiaCooldown>(source).is_some());

        // Verify event was sent
        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_cursor();
        let mut found = false;

        for event in reader.read(events) {
            if event.source == source && event.target == target {
                assert!((event.amount - PARANOIA_AFFINITY_PENALTY).abs() < f32::EPSILON);
                found = true;
            }
        }

        assert!(
            found,
            "Expected an AffinityChange event from source to target"
        );
    }

    #[test]
    fn test_paranoia_network_ignores_low_stress() {
        let mut world = setup_world();

        // Low stress pop (Source)
        let source = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        let _target = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 5 },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(paranoia_network_system).unwrap();

        assert!(world.get::<ParanoiaCooldown>(source).is_none());

        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_cursor();
        assert!(
            reader.read(events).next().is_none(),
            "No events should be sent for low stress"
        );
    }

    #[test]
    fn test_paranoia_network_respects_cooldown() {
        let mut world = setup_world();

        // High stress pop WITH cooldown active
        let source = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                StressTracker {
                    accumulated_stress: 95.0,
                },
                ParanoiaCooldown {
                    ticks_remaining: 10,
                },
            ))
            .id();

        let _target = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 5 },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(paranoia_network_system).unwrap();

        // Verify cooldown was decremented
        let cooldown = world.get::<ParanoiaCooldown>(source).unwrap();
        assert_eq!(cooldown.ticks_remaining, 9);

        // Verify NO event was sent
        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_cursor();
        assert!(
            reader.read(events).next().is_none(),
            "No events should be sent while on cooldown"
        );
    }
}
