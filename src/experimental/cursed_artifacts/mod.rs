//! Cursed Artifacts (Nova Feature)
//!
//! # The Spark
//! The Anomaly scanning system provides a flat amount of resources (e.g. Knowledge).
//! But what if some of these ruins held forbidden secrets?
//!
//! # The Feature
//! Existing `AnomalyType::Ruins` entities are occasionally tagged with `CursedAnomaly`.
//! When a Pop finishes exploring a `CursedAnomaly`, they receive a negative psychological
//! trait (like `VoidTouched` or `Spiteful`) and it dispatches an `AddChronicleEvent` to
//! log this tragic discovery into the colony's history.

use crate::layer1::anomalies::{Anomaly, AnomalyType, ScanProgress};
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component indicating an Anomaly is cursed.
#[derive(Component)]
pub struct CursedAnomaly;

/// Occasionally marks newly spawned or existing `Ruins` anomalies as cursed.
pub fn spawn_cursed_anomalies_system(
    mut commands: Commands,
    anomalies: Query<(Entity, &Anomaly), Without<CursedAnomaly>>,
) {
    let mut rng = rand::thread_rng();
    for (entity, anomaly) in anomalies.iter() {
        if anomaly.anomaly_type == AnomalyType::Ruins && rng.gen_bool(0.1) {
            commands.entity(entity).insert(CursedAnomaly);
        }
    }
}

/// System that applies curses to Pops who complete exploring a cursed anomaly.
#[allow(clippy::type_complexity)]
pub fn apply_curse_system(
    mut commands: Commands,
    mut pops: Query<(&MovementTarget, &mut Traits), (With<Pop>, With<AtTarget>)>,
    anomalies: Query<(Entity, &ScanProgress), (With<Anomaly>, With<CursedAnomaly>)>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    let mut rng = rand::thread_rng();

    for (target, mut traits) in pops.iter_mut() {
        if target.for_action != ActionType::Explore {
            continue;
        }

        if let Ok((entity, progress)) = anomalies.get(target.target_entity) {
            // If the scan is complete (or very close to complete)
            if progress.is_complete() {
                // Apply a curse
                let curse = if rng.gen_bool(0.5) {
                    Trait::VoidTouched
                } else {
                    Trait::Spiteful
                };

                if !traits.has(curse) {
                    traits.add(curse);

                    chronicle_events.send(AddChronicleEvent {
                        text: "A colonist unearthed a cursed artifact and their mind was forever altered by forbidden knowledge.".to_string(),
                        importance: EventImportance::Major,
            ..Default::default()});

                    // Remove the CursedAnomaly component so we don't curse them multiple times
                    commands.entity(entity).remove::<CursedAnomaly>();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_cursed_anomalies_tags_ruins() {
        let mut world = World::new();

        world.spawn(Anomaly {
            anomaly_type: AnomalyType::Ruins,
            reward_amount: 100.0,
        });

        world.spawn(Anomaly {
            anomaly_type: AnomalyType::Geode,
            reward_amount: 100.0,
        });

        // Run the system multiple times to ensure the 10% chance triggers for the ruin
        // (Mocking random here would be better but we can just run it until it hits for the test)
        let mut hit = false;
        for _ in 0..100 {
            let mut schedule = Schedule::default();
            schedule.add_systems(spawn_cursed_anomalies_system);
            schedule.run(&mut world);

            // Verify
            let mut query = world.query::<(&Anomaly, Option<&CursedAnomaly>)>();
            for (anomaly, cursed) in query.iter(&world) {
                if cursed.is_some() {
                    assert_eq!(anomaly.anomaly_type, AnomalyType::Ruins);
                    hit = true;
                }
            }
        }
        assert!(hit, "Ruins should eventually get cursed");
    }

    #[test]
    fn test_apply_curse_system() {
        let mut world = World::new();
        world.init_resource::<Events<AddChronicleEvent>>();

        let anomaly = world
            .spawn((
                Anomaly {
                    anomaly_type: AnomalyType::Ruins,
                    reward_amount: 100.0,
                },
                CursedAnomaly,
                ScanProgress {
                    current: 100.0,
                    required: 100.0,
                },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                Traits::default(),
                MovementTarget {
                    target_entity: anomaly,
                    target_position: Default::default(),
                    for_action: ActionType::Explore,
                },
                AtTarget,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_curse_system);
        schedule.run(&mut world);

        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::VoidTouched) || traits.has(Trait::Spiteful));

        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).count() > 0);
    }
}
