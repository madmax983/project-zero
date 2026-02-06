//! Experimental module for Pop dreams.
//!
//! Adds narrative depth by giving sleeping pops a chance to dream,
//! potentially generating knowledge (Inspiration).

use crate::experimental::biography::Biography;
use crate::layer1::execution::{AssignedTo, AssignmentType};
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::shared::log::MessageLog;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

/// A dream experienced by a Pop.
#[derive(Component, Debug, Clone)]
pub struct Dream {
    /// The content of the dream.
    pub content: String,
    /// The tick when this dream occurred.
    pub tick: u64,
}

/// System to generate dreams for sleeping pops.
pub fn dream_system(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;

    // Collect sleeping pops (those assigned to housing)
    // We clone biography data to avoid borrow checker issues when mutating world later
    let sleeping_pops: Vec<(Entity, Option<Biography>)> = world
        .query_filtered::<(Entity, Option<&Biography>, &AssignedTo), With<Pop>>()
        .iter(world)
        .filter(|(_, _, assigned)| assigned.assignment_type == AssignmentType::HousingResident)
        .map(|(e, bio, _)| (e, bio.cloned()))
        .collect();

    // Iterate and process dreams
    for (entity, bio) in sleeping_pops {
        let mut rng = rand::thread_rng();

        // 1% chance per tick to dream while sleeping
        if rng.gen_bool(0.01) {
            let dream_content = generate_dream(&mut rng, bio.as_ref());

            // Apply dream component
            world.entity_mut(entity).insert(Dream {
                content: dream_content.clone(),
                tick: current_tick,
            });

            // 10% chance for Inspiration (net 0.1% chance per tick while sleeping)
            if rng.gen_bool(0.10) {
                let amount = 5.0;
                let mut res = world.resource_mut::<ColonyResources>();
                res.knowledge = (res.knowledge + amount).clamp(0.0, res.max_knowledge);

                if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                    log.add(format!(
                        "Inspiration: A pop dreamed of '{}' and gained insight!",
                        dream_content
                    ));
                }
            } else if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                // Just log the dream for flavor
                log.add(format!("Dream: {}", dream_content));
            }
        }
    }
}

fn generate_dream(rng: &mut impl Rng, bio: Option<&Biography>) -> String {
    if let Some(bio) = bio
        && !bio.events.is_empty()
        && rng.gen_bool(0.7)
    {
        // Dream about past events
        // Safe unwrap because !is_empty check
        let event = bio.events.choose(rng).unwrap();
        format!("distorted memory of {}", event.text)
    } else {
        // Generic dreams
        let themes = [
            "flying over the colony",
            "endless fields of wheat",
            "a dark forest",
            "building a great monument",
            "falling forever",
            "eating a giant feast",
            "strange lights in the sky",
            "walking on water",
        ];
        (*themes.choose(rng).unwrap()).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::biography::BiographyEvent;

    #[test]
    fn test_dream_component() {
        let dream = Dream {
            content: "test".to_string(),
            tick: 100,
        };
        assert_eq!(dream.content, "test");
        assert_eq!(dream.tick, 100);
    }

    #[test]
    fn test_generate_dream_generic() {
        let mut rng = rand::thread_rng();
        let dream = generate_dream(&mut rng, None);
        assert!(!dream.is_empty());
    }

    #[test]
    fn test_generate_dream_biography() {
        let mut rng = rand::thread_rng();
        let bio = Biography {
            events: vec![BiographyEvent {
                tick: 0,
                text: "Event A".to_string(),
            }],
        };
        // Run multiple times to ensure coverage of random branch
        for _ in 0..20 {
            let dream = generate_dream(&mut rng, Some(&bio));
            assert!(!dream.is_empty());
            if dream.contains("distorted memory") {
                assert!(dream.contains("Event A"));
            }
        }
    }

    #[test]
    fn test_dream_system_adds_dream_component() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());

        // Spawn a sleeping pop
        let pop = world
            .spawn((
                Pop,
                AssignedTo {
                    entity: Entity::PLACEHOLDER,
                    assignment_type: AssignmentType::HousingResident,
                },
            ))
            .id();

        // Run system enough times to trigger probability
        let mut triggered = false;
        for _ in 0..1000 {
            dream_system(&mut world);
            if world.get::<Dream>(pop).is_some() {
                triggered = true;
                break;
            }
        }

        assert!(triggered, "Dream system should eventually trigger");
    }

    #[test]
    fn test_dream_system_ignores_awake_pops() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());

        // Spawn an awake pop (working)
        let pop = world
            .spawn((
                Pop,
                AssignedTo {
                    entity: Entity::PLACEHOLDER,
                    assignment_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        // Run system many times
        for _ in 0..100 {
            dream_system(&mut world);
        }

        assert!(
            world.get::<Dream>(pop).is_none(),
            "Awake pop should not dream"
        );
    }
}
