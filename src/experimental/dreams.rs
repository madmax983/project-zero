//! Experimental module for Pop dreams.
//!
//! Adds narrative depth by giving sleeping pops a chance to dream,
//! potentially generating knowledge (Inspiration).

use crate::experimental::biography::Biography;
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::shared::log::MessageLog;
use crate::shared::narrative::NarrativeGenerator;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

/// A dream experienced by a Pop.
#[derive(Component, Debug, Clone)]
pub struct Dream {
    /// The content of the dream.
    pub content: String,
    /// The tick when this dream occurred.
    pub tick: u64,
}

/// System to generate dreams for sleeping pops.
pub fn dream_system(
    time: Res<SimulationTime>,
    sleeping_pops: Query<(Entity, Option<&Biography>, &AssignedTo), With<Pop>>,
    mut resources: ResMut<ColonyResources>,
    mut log: Option<ResMut<MessageLog>>,
    mut commands: Commands,
    generator: Res<NarrativeGenerator>,
) {
    let current_tick = time.tick;

    for (entity, bio, assigned) in &sleeping_pops {
        if assigned.assignment_type != AssignmentType::HousingResident {
            continue;
        }

        let mut rng = rand::thread_rng();

        // 1% chance per tick to dream while sleeping
        if rng.gen_bool(0.01) {
            let dream_content = generate_dream(&mut rng, bio, &generator);

            commands.entity(entity).insert(Dream {
                content: dream_content.clone(),
                tick: current_tick,
            });

            // 10% chance for Inspiration (net 0.1% chance per tick while sleeping)
            if rng.gen_bool(0.10) {
                let amount = 5.0;
                resources.knowledge =
                    (resources.knowledge + amount).clamp(0.0, resources.max_knowledge);

                if let Some(ref mut log) = log {
                    log.add(format!(
                        "Inspiration: A pop dreamed of '{dream_content}' and gained insight!"
                    ));
                }
            } else if let Some(ref mut log) = log {
                log.add(format!("Dream: {dream_content}"));
            }
        }
    }
}

fn generate_dream(
    rng: &mut impl Rng,
    bio: Option<&Biography>,
    generator: &NarrativeGenerator,
) -> String {
    if let Some(bio) = bio
        && !bio.events.is_empty()
        && rng.gen_bool(0.7)
    {
        // Dream about past events
        let event = bio.events.choose(rng).unwrap();
        format!("distorted memory of {}", event.text)
    } else {
        // Fragment-based dreams from varied categories
        let categories = [
            "VOID_ANOMALY",
            "EMOTIONAL_WEIGHT",
            "MEMORY_TOPIC",
            "CATASTROPHE_TYPE",
            "PLACE_DESCRIPTOR",
        ];
        let category = categories[rng.gen_range(0..categories.len())];
        generator
            .get_random_fragment(category)
            .cloned()
            .unwrap_or_else(|| "strange lights in the sky".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::biography::BiographyEvent;
    use crate::shared::narrative::NarrativeGenerator;
    use bevy_ecs::system::RunSystemOnce;

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
        let narrator = NarrativeGenerator::from_embedded();
        let dream = generate_dream(&mut rng, None, &narrator);
        assert!(!dream.is_empty());
    }

    #[test]
    fn test_generate_dream_biography() {
        let mut rng = rand::thread_rng();
        let narrator = NarrativeGenerator::from_embedded();
        let bio = Biography {
            events: vec![BiographyEvent {
                tick: 0,
                text: "Event A".to_string(),
            }],
        };
        for _ in 0..20 {
            let dream = generate_dream(&mut rng, Some(&bio), &narrator);
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
        world.insert_resource(NarrativeGenerator::from_embedded());

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
            world.run_system_once(dream_system).unwrap();
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
        world.insert_resource(NarrativeGenerator::from_embedded());

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
            world.run_system_once(dream_system).unwrap();
        }

        assert!(
            world.get::<Dream>(pop).is_none(),
            "Awake pop should not dream"
        );
    }
}
