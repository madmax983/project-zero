use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::resources::ColonyResources;
use crate::layer1::traits::{Trait, Traits};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component marker for Observatory buildings.
#[derive(Component)]
pub struct Observatory {
    /// Efficiency of the observatory (e.g., 100.0 is normal).
    pub efficiency: f32,
}

impl Default for Observatory {
    fn default() -> Self {
        Self { efficiency: 100.0 }
    }
}

/// Processes logic for Pops assigned to Observatories.
///
/// 1. Generates 0.02 Knowledge per pop per tick.
/// 2. Has a 1% chance per tick to trigger "The Overview Effect" (Mood Modifier).
///    - 50% "Cosmic Inspiration" (+0.15)
///    - 50% "Existential Dread" (-0.10)
pub fn process_observe_system(
    mut pops: Query<(Entity, &AssignedTo, &mut Morale, Option<&Traits>)>,
    observatories: Query<&Observatory>,
    mut resources: ResMut<ColonyResources>,
    mut log: Option<ResMut<MessageLog>>,
) {
    let mut rng = rand::thread_rng();

    for (_entity, assignment, mut morale, traits) in &mut pops {
        if assignment.assignment_type == AssignmentType::ObservatoryWorker {
            if let Ok(observatory) = observatories.get(assignment.entity) {
                // 1. Generate Knowledge
                let gain = 0.02 * (observatory.efficiency / 100.0);
                resources.knowledge += gain;
                resources.knowledge = resources.knowledge.clamp(0.0, resources.max_knowledge);

                // 2. Chance for Overview Effect (1% per tick)
                if rng.gen_bool(0.01) {
                    let mut inspiration_chance: f64 = 0.5;

                    if let Some(traits) = traits {
                        if traits.has(Trait::Optimist) || traits.has(Trait::Curious) {
                            inspiration_chance += 0.3;
                        }
                        if traits.has(Trait::Anxious) || traits.has(Trait::Traditionalist) {
                            inspiration_chance -= 0.3;
                        }
                    }

                    // Clamp just in case (0.2 to 0.8 range typically)
                    inspiration_chance = inspiration_chance.clamp(0.1, 0.9);

                    if rng.gen_bool(inspiration_chance) {
                        morale.add_modifier(MoodModifier {
                            label: "Cosmic Inspiration".to_string(),
                            value: 0.15,
                            duration: 500,
                        });
                        if let Some(log) = &mut log {
                            log.add("A colonist was inspired by the cosmos!");
                        }
                    } else {
                        morale.add_modifier(MoodModifier {
                            label: "Existential Dread".to_string(),
                            value: -0.10,
                            duration: 500,
                        });
                        if let Some(log) = &mut log {
                            log.add("A colonist stared into the void...");
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{process_observe_system, Observatory};
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::building::BuildingType;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::tech::Tech;
    use crate::layer1::traits::{Trait, Traits};
    use crate::shared::log::MessageLog;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_astronomy_tech_exists() {
        // Just verifying variant exists
        let _ = Tech::Astronomy;
        assert_eq!(Tech::Astronomy.cost(), 50.0); // Expensive late-game tech
    }

    #[test]
    fn test_observatory_requires_astronomy() {
        assert_eq!(
            BuildingType::Observatory.required_tech(),
            Some(Tech::Astronomy)
        );
    }

    #[test]
    fn test_observe_action_generates_knowledge() {
        let mut world = World::new();

        // Setup Resources
        let res = ColonyResources {
            knowledge: 0.0,
            max_knowledge: 100.0,
            ..Default::default()
        };
        world.insert_resource(res);

        // Setup Observatory
        let observatory = world.spawn(Observatory::default()).id();

        // Setup Pop working there
        world.spawn((
            Pop,
            AssignedTo {
                entity: observatory,
                assignment_type: AssignmentType::ObservatoryWorker,
            },
            Morale::default(), // Needed for query
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);
        schedule.run(&mut world);

        // Check Knowledge Gain
        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge > 0.0);
    }

    #[test]
    fn test_observe_action_applies_mood_modifier() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let observatory = world.spawn(Observatory::default()).id();
        let pop = world
            .spawn((
                Pop,
                Morale::default(),
                AssignedTo {
                    entity: observatory,
                    assignment_type: AssignmentType::ObservatoryWorker,
                },
            ))
            .id();

        // Run system multiple times to ensure probability triggers
        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);

        // Mock RNG or run enough times to trigger effect
        // 1% chance.
        for _ in 0..2000 {
            schedule.run(&mut world);
        }

        let morale = world.get::<Morale>(pop).unwrap();
        let has_inspired = morale
            .modifiers
            .iter()
            .any(|m| m.label == "Cosmic Inspiration");
        let has_dread = morale
            .modifiers
            .iter()
            .any(|m| m.label == "Existential Dread");

        assert!(
            has_inspired || has_dread,
            "Should have triggered a mood modifier"
        );
    }

    #[test]
    fn test_optimist_gets_more_inspiration() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        let observatory = world.spawn(Observatory::default()).id();

        // Spawn many pops to get statistical significance faster
        let pop_count = 100;
        let mut pops = Vec::new();
        for _ in 0..pop_count {
            pops.push(
                world
                    .spawn((
                        Pop,
                        Morale::default(),
                        {
                            let mut t = Traits::default();
                            t.add(Trait::Optimist);
                            t
                        },
                        AssignedTo {
                            entity: observatory,
                            assignment_type: AssignmentType::ObservatoryWorker,
                        },
                    ))
                    .id(),
            );
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);

        // Run for enough ticks to get events
        for _ in 0..200 {
            schedule.run(&mut world);
        }

        let mut inspired_count = 0;
        let mut dread_count = 0;

        for pop in pops {
            let morale = world.get::<Morale>(pop).unwrap();
            for modifier in &morale.modifiers {
                if modifier.label == "Cosmic Inspiration" {
                    inspired_count += 1;
                } else if modifier.label == "Existential Dread" {
                    dread_count += 1;
                }
            }
        }

        assert!(
            inspired_count > dread_count * 2,
            "Optimists should have significantly more inspiration than dread. Got Inspired: {}, Dread: {}",
            inspired_count,
            dread_count
        );
    }

    #[test]
    fn test_anxious_gets_more_dread() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        let observatory = world.spawn(Observatory::default()).id();

        // Spawn many pops
        let pop_count = 100;
        let mut pops = Vec::new();
        for _ in 0..pop_count {
            pops.push(
                world
                    .spawn((
                        Pop,
                        Morale::default(),
                        {
                            let mut t = Traits::default();
                            t.add(Trait::Anxious);
                            t
                        },
                        AssignedTo {
                            entity: observatory,
                            assignment_type: AssignmentType::ObservatoryWorker,
                        },
                    ))
                    .id(),
            );
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);

        for _ in 0..200 {
            schedule.run(&mut world);
        }

        let mut inspired_count = 0;
        let mut dread_count = 0;

        for pop in pops {
            let morale = world.get::<Morale>(pop).unwrap();
            for modifier in &morale.modifiers {
                if modifier.label == "Cosmic Inspiration" {
                    inspired_count += 1;
                } else if modifier.label == "Existential Dread" {
                    dread_count += 1;
                }
            }
        }

        assert!(
            dread_count > inspired_count * 2,
            "Anxious pops should have significantly more dread than inspiration. Got Inspired: {}, Dread: {}",
            inspired_count,
            dread_count
        );
    }

    #[test]
    fn test_observatory_logs_message() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default()); // Add MessageLog
        let observatory = world.spawn(Observatory::default()).id();

        world.spawn((
            Pop,
            Morale::default(),
            AssignedTo {
                entity: observatory,
                assignment_type: AssignmentType::ObservatoryWorker,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);

        // Run until log appears
        let mut log_found = false;
        for _ in 0..5000 {
            schedule.run(&mut world);
            let log = world.resource::<MessageLog>();
            if !log.messages.is_empty() {
                log_found = true;
                let msg = &log.messages.back().unwrap().text;
                assert!(
                    msg.contains("inspired by the cosmos") || msg.contains("stared into the void") || msg.contains("discovered a comet"),
                    "Unexpected log message: {}",
                    msg
                );
                break;
            }
        }

        assert!(log_found, "Should have logged a message");
    }
}
