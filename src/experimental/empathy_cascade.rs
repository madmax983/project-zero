//! The Empathy Cascade (Nova Feature).
//!
//! # The Spark
//! We have a `Health` system and an `EmpathicLink` trait. What if pain was contagious?
//!
//! # The Feature
//! Pops with the `EmpathicLink` trait broadcast their physical trauma. When they take
//! damage, they radiate a pulse of stress to all nearby Pops. If the nearby Pops also
//! possess the `EmpathicLink` trait, they don't just get stressed—they suffer sympathetic
//! physical damage. This creates a terrifying chain reaction ("Cascade") where a single
//! injury in a crowded room of empaths can result in mass casualties.

use crate::layer1::biology::health::Health;
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Pop, PopName};
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const CASCADE_RADIUS: u32 = 5;
const STRESS_PER_DAMAGE_POINT: f32 = 2.0;
const SYMPATHETIC_DAMAGE_MULTIPLIER: f32 = 0.5;

/// Component to track health from the previous tick so we can detect damage.
#[derive(Component, Default, Clone)]
pub struct EmpathicHealthTracker {
    pub previous_health: f32,
}

/// System that initializes tracking for Empathic pops
#[allow(clippy::type_complexity)]
pub fn init_empathic_health_tracker_system(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Traits), (With<Pop>, Without<EmpathicHealthTracker>)>,
) {
    for (entity, health, traits) in query.iter() {
        if traits.has(Trait::EmpathicLink) {
            commands.entity(entity).insert(EmpathicHealthTracker {
                previous_health: health.current,
            });
        }
    }
}

/// System that monitors Empathic Pops for damage and triggers the cascade.
#[allow(clippy::type_complexity)]
pub fn empathy_cascade_system(
    mut query: Query<
        (
            Entity,
            &GridPosition,
            &mut Health,
            &Traits,
            &mut EmpathicHealthTracker,
            &mut StressTracker,
            Option<&PopName>,
        ),
        With<Pop>,
    >,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    // Pass 1: Find all bursts of empathic damage
    let mut bursts = Vec::new();

    // We only iterate to find the damage first
    for (entity, pos, health, _, mut tracker, _, name) in query.iter_mut() {
        let damage_taken = tracker.previous_health - health.current;
        if damage_taken > 0.0 {
            bursts.push((*pos, damage_taken, entity, name.map(|n| n.0.clone())));
        }
        // Update tracker for next tick. Sympathetic damage taken in Pass 2 will be caught next tick, causing a cascade!
        tracker.previous_health = health.current;
    }

    if bursts.is_empty() {
        return;
    }

    let mut cascade_occurred = false;
    let mut primary_victim_name = None;

    // Pass 2: Apply the bursts to other Pops
    for (source_pos, damage, source_entity, name_opt) in bursts {
        let mut local_cascade = false;

        // We use iter_combinations_mut but it's easier to just iterate all and check distance against the known bursts.
        // Wait, iter_mut() again on the same query is fine as long as we dropped the first iter.
        for (
            target_entity,
            target_pos,
            mut target_health,
            target_traits,
            _,
            mut target_stress,
            _,
        ) in query.iter_mut()
        {
            if source_entity == target_entity {
                continue;
            }

            if source_pos.distance_chebyshev(*target_pos) <= CASCADE_RADIUS {
                // Anyone nearby takes stress
                target_stress.accumulated_stress += damage * STRESS_PER_DAMAGE_POINT;

                // Empaths take actual damage
                if target_traits.has(Trait::EmpathicLink) {
                    target_health.take_damage(damage * SYMPATHETIC_DAMAGE_MULTIPLIER);
                    local_cascade = true;
                }
            }
        }

        if local_cascade && !cascade_occurred {
            cascade_occurred = true;
            primary_victim_name = name_opt;
        }
    }

    if cascade_occurred {
        let display_name = primary_victim_name.unwrap_or_else(|| "An empath".to_string());
        chronicle_events.send(AddChronicleEvent {
            text: format!("{} suffered a terrible injury, unleashing a violent wave of sympathetic pain that ravaged nearby empaths.", display_name),
            importance: EventImportance::Major,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_empathy_cascade() {
        let mut world = World::new();
        world.init_resource::<Events<AddChronicleEvent>>();

        let mut traits_empath = Traits::default();
        traits_empath.add(Trait::EmpathicLink);

        // Source pop (gets hurt)
        let _pop_source = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                Health {
                    current: 50.0,
                    max: 100.0,
                }, // Damaged down to 50 from 100
                traits_empath.clone(),
                EmpathicHealthTracker {
                    previous_health: 100.0,
                },
                StressTracker::default(),
                PopName("Alpha".to_string()),
            ))
            .id();

        // Target empath (should take damage and stress)
        let pop_target_empath = world
            .spawn((
                Pop,
                GridPosition { x: 12, y: 10 }, // Distance 2
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                traits_empath.clone(),
                EmpathicHealthTracker {
                    previous_health: 100.0,
                },
                StressTracker::default(),
                PopName("Beta".to_string()),
            ))
            .id();

        // Target normal (should only take stress)
        let pop_target_normal = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 12 }, // Distance 2
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                Traits::default(),
                EmpathicHealthTracker {
                    previous_health: 100.0,
                }, // Has tracker just for ease of querying in test
                StressTracker::default(),
                PopName("Gamma".to_string()),
            ))
            .id();

        // Target empath too far away (should be unaffected)
        let pop_target_far = world
            .spawn((
                Pop,
                GridPosition { x: 50, y: 50 }, // Distance 40
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                traits_empath.clone(),
                EmpathicHealthTracker {
                    previous_health: 100.0,
                },
                StressTracker::default(),
                PopName("Delta".to_string()),
            ))
            .id();

        world.run_system_once(empathy_cascade_system).unwrap();

        // Source took 50 damage.
        // Sympathetic damage = 50 * 0.5 = 25
        // Stress = 50 * 2.0 = 100

        let target_empath_health = world.get::<Health>(pop_target_empath).unwrap();
        assert_eq!(target_empath_health.current, 75.0); // 100 - 25

        let target_empath_stress = world.get::<StressTracker>(pop_target_empath).unwrap();
        assert_eq!(target_empath_stress.accumulated_stress, 100.0);

        let target_normal_health = world.get::<Health>(pop_target_normal).unwrap();
        assert_eq!(target_normal_health.current, 100.0); // No damage

        let target_normal_stress = world.get::<StressTracker>(pop_target_normal).unwrap();
        assert_eq!(target_normal_stress.accumulated_stress, 100.0); // Took stress

        let target_far_health = world.get::<Health>(pop_target_far).unwrap();
        assert_eq!(target_far_health.current, 100.0);

        let target_far_stress = world.get::<StressTracker>(pop_target_far).unwrap();
        assert_eq!(target_far_stress.accumulated_stress, 0.0);

        // Check events
        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let chronicle_events: Vec<_> = reader.read(events).collect();
        assert!(!chronicle_events.is_empty(), "Should emit chronicle event");
        assert!(
            chronicle_events[0].text.contains("Alpha"),
            "Event should contain the primary victim's name"
        );
    }

    #[test]
    fn test_init_tracker() {
        let mut world = World::new();
        let mut traits_empath = Traits::default();
        traits_empath.add(Trait::EmpathicLink);

        let pop = world
            .spawn((
                Pop,
                Health {
                    current: 80.0,
                    max: 100.0,
                },
                traits_empath,
            ))
            .id();

        world
            .run_system_once(init_empathic_health_tracker_system)
            .unwrap();

        let tracker = world.get::<EmpathicHealthTracker>(pop).unwrap();
        assert_eq!(tracker.previous_health, 80.0);
    }
}
