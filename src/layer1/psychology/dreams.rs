//! Experimental module for Pop dreams.
//!
//! Adds narrative depth by giving sleeping pops a chance to dream about the colony's history
//! and their own memories, affecting their leisure and potentially generating knowledge.

use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::shared::log::MessageLog;
use crate::shared::narrative::NarrativeGenerator;
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
    /// The impact on leisure (positive or negative).
    pub impact: f32,
    /// Whether it was a nightmare.
    pub is_nightmare: bool,
}

/// Stores the dream history of a Pop.
#[derive(Component, Debug, Default, Clone)]
pub struct DreamJournal {
    /// The most recent dream.
    pub last_dream: Option<Dream>,
    /// History of past dreams (capped).
    pub history: Vec<Dream>,
}

/// Marker component indicating a pop has already dreamt during this sleep cycle.
#[derive(Component)]
pub struct DreamtThisSleep;

/// System to generate dreams for sleeping pops.
#[allow(clippy::type_complexity)]
pub fn dream_system(
    time: Res<SimulationTime>,
    mut sleeping_pops: Query<
        (
            Entity,
            &AssignedTo,
            &PopAction,
            &mut Needs,
            Option<&Memories>,
            Option<&mut DreamJournal>,
        ),
        (With<Pop>, Without<DreamtThisSleep>),
    >,
    chronicle: Res<Chronicle>,
    mut resources: ResMut<ColonyResources>,
    mut log: Option<ResMut<MessageLog>>,
    mut commands: Commands,
    generator: Res<NarrativeGenerator>,
) {
    let current_tick = time.tick;
    let chronicle_events = &chronicle.events;
    let mut rng = rand::thread_rng();

    for (entity, assigned, action, mut needs, memories_opt, journal_opt) in &mut sleeping_pops {
        // Must be sleeping in a bed
        if assigned.assignment_type != AssignmentType::HousingResident
            || action.current != ActionType::SatisfyRest
        {
            continue;
        }

        // 1% chance per tick to dream -> eventually happens during sleep
        if !rng.gen_bool(0.01) {
            continue;
        }

        // Mark as dreamt immediately to prevent multiple dreams per sleep
        commands.entity(entity).insert(DreamtThisSleep);

        let (dream_content, impact, is_nightmare) =
            generate_dream_content(&mut rng, chronicle_events, &generator, memories_opt);

        // Apply dream impact
        // Nightmares reduce leisure (stress), Good dreams increase it
        needs.leisure = (needs.leisure + impact).clamp(0.0, 1.0);

        let dream = Dream {
            content: dream_content.clone(),
            tick: current_tick,
            impact,
            is_nightmare,
        };

        // Update Journal
        if let Some(mut journal) = journal_opt {
            journal.history.push(dream.clone());
            if journal.history.len() > 10 {
                journal.history.remove(0);
            }
            journal.last_dream = Some(dream);
        } else {
            // Create new journal if missing
            let journal = DreamJournal {
                last_dream: Some(dream.clone()),
                history: vec![dream],
            };
            commands.entity(entity).insert(journal);
        }

        // Inspiration Chance (only on good dreams)
        if !is_nightmare && rng.gen_bool(0.05) {
            let knowledge_gain = 5.0;
            resources.knowledge =
                (resources.knowledge + knowledge_gain).clamp(0.0, resources.max_knowledge);

            if let Some(ref mut log) = log {
                log.add(format!(
                    "Inspiration: A pop had a vision of '{dream_content}' and gained insight!"
                ));
            }
        } else if let Some(ref mut log) = log {
            // Log vivid dreams or nightmares
            if is_nightmare || impact.abs() > 0.1 {
                let prefix = if is_nightmare { "Nightmare" } else { "Dream" };
                log.add(format!("{prefix}: {dream_content}"));
            }
        }
    }
}

/// Removes the `DreamtThisSleep` marker when the pop wakes up.
pub fn cleanup_dream_marker_system(
    mut commands: Commands,
    mut query: Query<(Entity, &PopAction), With<DreamtThisSleep>>,
) {
    for (entity, action) in &mut query {
        // If no longer sleeping, remove marker
        if action.current != ActionType::SatisfyRest {
            commands.entity(entity).remove::<DreamtThisSleep>();
        }
    }
}

fn generate_dream_content(
    rng: &mut impl Rng,
    chronicle_events: &[crate::layer1::chronicle::ChronicleEvent],
    generator: &NarrativeGenerator,
    memories_opt: Option<&Memories>,
) -> (String, f32, bool) {
    // 1. Check for Trauma/Memories (50% chance if memories exist)
    if let Some(memories) = memories_opt {
        // Collapsible if is fine here for readability of the let-else
        #[allow(clippy::collapsible_if)]
        if !memories.items.is_empty() && rng.gen_bool(0.5) {
            // Pick a random memory, weighted by intensity?
            // For now just random.
            if let Some(memory) = memories.items.choose(rng) {
                let (text, impact, nightmare) = interpret_memory(memory.memory_type);
                // Intensity scales the impact
                return (text, impact * memory.intensity, nightmare);
            }
        }
    }

    // 2. History (30% chance)
    if !chronicle_events.is_empty() && rng.gen_bool(0.3) {
        if let Some(event) = chronicle_events.choose(rng) {
            return match event.importance {
                EventImportance::Legendary | EventImportance::Major => {
                    (format!("relived the glory of: {}", event.text), 0.2, false)
                }
                EventImportance::Standard => (format!("recalled: {}", event.text), 0.05, false),
                EventImportance::Minor => {
                    (format!("faintly remembered: {}", event.text), 0.0, false)
                }
            };
        }
    }

    // Fallback logic
    {
        // 3. Abstract / Random
        let categories = [
            "VOID_ANOMALY",
            "EMOTIONAL_WEIGHT",
            "CATASTROPHE_TYPE",
            "PLACE_DESCRIPTOR",
        ];
        let category = categories[rng.gen_range(0..categories.len())];
        let content = generator
            .get_random_fragment(category)
            .cloned()
            .unwrap_or_else(|| "strange lights in the sky".to_string());

        // Randomly decide if it's a nightmare (10%)
        if rng.gen_bool(0.1) {
            (format!("nightmare of {content}"), -0.1, true)
        } else {
            (format!("dreamed of {content}"), 0.05, false)
        }
    }
}

fn interpret_memory(memory_type: MemoryType) -> (String, f32, bool) {
    match memory_type {
        MemoryType::WitnessedDeath => ("nightmare of a friend dying".to_string(), -0.2, true),
        MemoryType::StarvationTrauma => ("dreamed of gnawing hunger".to_string(), -0.15, true),
        MemoryType::DisgustedByVermin => ("dreamed of crawling insects".to_string(), -0.1, true),
        MemoryType::SawCorpse => ("nightmare of a dead body".to_string(), -0.1, true),
        MemoryType::WonFight => ("dreamed of victory in battle".to_string(), 0.1, false),
        MemoryType::AteFineMeal => ("dreamed of a delicious feast".to_string(), 0.1, false),
        MemoryType::AttendedFuneral => ("dreamed of saying goodbye".to_string(), 0.05, false),
        MemoryType::AdmiredArt => ("dreamed of beautiful art".to_string(), 0.1, false),
        MemoryType::SleptInAwfulRoom => {
            ("tossed and turned in a cold room".to_string(), -0.05, true)
        }
        MemoryType::SleptInLegendaryRoom => ("rested in a palace of gold".to_string(), 0.2, false),
        _ => ("dreamed of daily life".to_string(), 0.0, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::Chronicle;
    use crate::layer1::memory::{ActiveMemory, MemoryType};
    use crate::shared::narrative::NarrativeGenerator;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_dream_system_adds_journal() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());
        world.insert_resource(NarrativeGenerator::from_embedded());
        world.insert_resource(Chronicle::default());

        let pop = world
            .spawn((
                Pop,
                AssignedTo {
                    entity: Entity::PLACEHOLDER,
                    assignment_type: AssignmentType::HousingResident,
                },
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                Needs::default(),
            ))
            .id();

        // Run until dream triggers
        let mut triggered = false;
        for _ in 0..1000 {
            world.run_system_once(dream_system).unwrap();
            if world.get::<DreamJournal>(pop).is_some() {
                triggered = true;
                break;
            }
        }
        assert!(triggered, "Dream system should add DreamJournal");
        assert!(
            world.get::<DreamtThisSleep>(pop).is_some(),
            "Should be marked as dreamt"
        );
    }

    #[test]
    fn test_dream_memory_influence() {
        let mut rng = rand::thread_rng();
        let generator = NarrativeGenerator::from_embedded();
        let events = vec![];

        let mut memories = Memories::default();
        memories.items.push(ActiveMemory {
            memory_type: MemoryType::WitnessedDeath,
            added_at: 0,
            intensity: 1.0,
            forged: false,
        });

        // Loop until we hit the memory case (probabilistic)
        let mut hit_memory = false;
        for _ in 0..100 {
            let (content, impact, nightmare) =
                generate_dream_content(&mut rng, &events, &generator, Some(&memories));

            if content.contains("nightmare of a friend dying") {
                assert!(nightmare);
                assert!(impact < 0.0);
                hit_memory = true;
                break;
            }
        }
        assert!(
            hit_memory,
            "Should eventually dream about the traumatic memory"
        );
    }

    #[test]
    fn test_cleanup_marker() {
        let mut world = World::new();

        // Pop 1: Still sleeping -> Should keep marker
        let p1 = world
            .spawn((
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                DreamtThisSleep,
            ))
            .id();

        // Pop 2: Woke up -> Should lose marker
        let p2 = world
            .spawn((
                PopAction {
                    current: ActionType::Work,
                    ..Default::default()
                },
                DreamtThisSleep,
            ))
            .id();

        world.run_system_once(cleanup_dream_marker_system).unwrap();

        assert!(world.get::<DreamtThisSleep>(p1).is_some());
        assert!(world.get::<DreamtThisSleep>(p2).is_none());
    }
}
