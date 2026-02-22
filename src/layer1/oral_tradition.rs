//! System for Oral Tradition and Folk Tales.
//!
//! Converts colony history (Chronicle) into living legends that are shared in taverns.
//! Stories evolve over time, gaining mutations and providing buffs to listeners.

use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::layer1::needs::Needs;
use crate::layer1::social::Tavern;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

/// A story that has evolved from a historical event.
#[derive(Debug, Clone)]
pub struct Story {
    /// The current text of the story.
    pub text: String,
    /// The original tick when the event happened.
    pub origin_tick: u64,
    /// How many times the story has mutated.
    pub mutations: u32,
    /// The type/genre of the story.
    pub genre: StoryGenre,
}

/// Genre of a story, determining its effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryGenre {
    /// Heroic tales boost leisure/morale.
    Heroic,
    /// Tragedies build empathy (social bonding - placeholder).
    Tragedy,
    /// Warnings regarding safety/danger.
    Cautionary,
    /// Just interesting facts.
    Trivial,
}

/// Resource storing the collective oral tradition of the colony.
#[derive(Resource, Default, Debug)]
pub struct OralTradition {
    /// The collection of known stories.
    pub stories: Vec<Story>,
    /// The tick of the last processed chronicle event.
    pub last_processed_tick: u64,
}

impl OralTradition {
    /// Adds a story if it's not a duplicate (based on origin tick).
    pub fn add_story(&mut self, story: Story) {
        if !self
            .stories
            .iter()
            .any(|s| s.origin_tick == story.origin_tick)
        {
            self.stories.push(story);
        }
    }
}

/// System to convert new Chronicle events into Stories.
pub fn collect_chronicles_system(mut tradition: ResMut<OralTradition>, chronicle: Res<Chronicle>) {
    // Only look at events since last check
    let new_events: Vec<_> = chronicle
        .events
        .iter()
        .filter(|e| e.tick > tradition.last_processed_tick)
        .collect();

    if new_events.is_empty() {
        return;
    }

    // Update tracker
    if let Some(last) = new_events.last() {
        tradition.last_processed_tick = last.tick;
    }

    for event in new_events {
        // Heuristic for genre
        let genre = if event.text.to_lowercase().contains("died")
            || event.text.to_lowercase().contains("death")
        {
            StoryGenre::Tragedy
        } else if event.text.to_lowercase().contains("collapsed")
            || event.text.to_lowercase().contains("starvation")
        {
            StoryGenre::Cautionary
        } else {
            match event.importance {
                EventImportance::Legendary => StoryGenre::Heroic,
                EventImportance::Major => StoryGenre::Heroic,
                EventImportance::Standard => StoryGenre::Trivial, // Most standard events are just "X happened"
                EventImportance::Minor => StoryGenre::Trivial,
            }
        };

        if genre == StoryGenre::Trivial {
            // Keep some trivial ones for flavor, but maybe skip most?
            // For now, keep them.
        }

        let story = Story {
            text: event.text.clone(),
            origin_tick: event.tick,
            mutations: 0,
            genre,
        };

        tradition.add_story(story);
    }
}

/// System where pops in taverns tell stories to each other.
pub fn storytelling_system(
    mut tradition: ResMut<OralTradition>,
    mut tavern_query: Query<&Tavern>,
    mut pop_query: Query<&mut Needs>,
    mut log: ResMut<MessageLog>,
) {
    let mut rng = rand::thread_rng();

    // 10% chance per tavern per tick to tell a story
    if !rng.gen_bool(0.1) || tradition.stories.is_empty() {
        return;
    }

    for tavern in &mut tavern_query {
        if tavern.visitors.len() < 2 {
            continue;
        }

        // Pick a story
        // We use an index to mutate it in place later
        let story_idx = rng.gen_range(0..tradition.stories.len());
        let story = &mut tradition.stories[story_idx];

        // Apply effects to all visitors
        for &visitor in &tavern.visitors {
            if let Ok(mut needs) = pop_query.get_mut(visitor) {
                match story.genre {
                    StoryGenre::Heroic => {
                        needs.leisure = (needs.leisure + 0.1).min(1.0);
                    }
                    StoryGenre::Tragedy => {
                        // "Catharsis" - small leisure boost
                        needs.leisure = (needs.leisure + 0.05).min(1.0);
                    }
                    StoryGenre::Cautionary => {
                        // Increase wakefulness (fear keeps you awake)
                        needs.rest = (needs.rest + 0.05).min(1.0);
                    }
                    StoryGenre::Trivial => {}
                }
            }
        }

        // Mutation: 20% chance
        if rng.gen_bool(0.2) {
            mutate_story(story, &mut rng);
            log.add(format!("A legend evolves: '{}'", story.text));
        }
    }
}

fn mutate_story(story: &mut Story, rng: &mut impl Rng) {
    let suffixes = [
        " It is known.",
        " So they say.",
        " Or was it?",
        " The spirits were watching.",
        " And the colony survived.",
        " Beware the void.",
    ];

    let replacements = [
        ("colony", "Homeland"),
        ("Colony", "Homeland"),
        ("built", "forged"),
        ("died", "returned to the void"),
        ("fire", "The Cleansing Flame"),
        ("storm", "The Breath of Giants"),
        ("founded", "birthed from chaos"),
    ];

    if rng.gen_bool(0.5) {
        // Append suffix
        let suffix = suffixes.choose(rng).unwrap();
        if !story.text.ends_with(suffix) {
            story.text.push_str(suffix);
        }
    } else {
        // Replace word
        let (target, replacement) = replacements.choose(rng).unwrap();
        // Case insensitive replacement would be better but simple replace is fine for MVP
        story.text = story.text.replace(target, replacement);
    }

    story.mutations += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::{Chronicle, EventImportance};
    use crate::layer1::needs::Needs;
    use crate::layer1::social::Tavern;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_collect_chronicles() {
        let mut world = World::new();
        world.insert_resource(OralTradition::default());
        let mut chronicle = Chronicle::default();
        chronicle.add_event(100, "Heroic Deed".to_string(), EventImportance::Legendary);
        world.insert_resource(chronicle);

        world.run_system_once(collect_chronicles_system).unwrap();

        let tradition = world.resource::<OralTradition>();
        assert_eq!(tradition.stories.len(), 1);
        assert_eq!(tradition.stories[0].genre, StoryGenre::Heroic);
    }

    #[test]
    fn test_collect_chronicles_genre_detection() {
        let mut world = World::new();
        world.insert_resource(OralTradition::default());
        let mut chronicle = Chronicle::default();
        chronicle.add_event(100, "Someone died".to_string(), EventImportance::Standard);
        world.insert_resource(chronicle);

        world.run_system_once(collect_chronicles_system).unwrap();

        let tradition = world.resource::<OralTradition>();
        assert_eq!(tradition.stories[0].genre, StoryGenre::Tragedy);
    }

    #[test]
    fn test_storytelling_buffs() {
        let mut world = World::new();
        let mut tradition = OralTradition::default();
        tradition.add_story(Story {
            text: "Heroic Tale".to_string(),
            origin_tick: 1,
            mutations: 0,
            genre: StoryGenre::Heroic,
        });
        world.insert_resource(tradition);
        world.insert_resource(MessageLog::default());

        let pop = world
            .spawn(Needs {
                leisure: 0.5,
                ..Default::default()
            })
            .id();

        let mut tavern = Tavern::default();
        tavern.visitors.push(pop);
        tavern.visitors.push(Entity::PLACEHOLDER); // Need 2 visitors
        world.spawn(tavern);

        // Run system enough times to trigger probability
        for _ in 0..50 {
            world.run_system_once(storytelling_system).unwrap();
        }

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure > 0.5, "Heroic story should boost leisure");
    }

    #[test]
    fn test_mutation() {
        let mut rng = rand::thread_rng();
        let mut story = Story {
            text: "The colony was founded.".to_string(),
            origin_tick: 0,
            mutations: 0,
            genre: StoryGenre::Trivial,
        };

        // Force mutation
        for _ in 0..10 {
            mutate_story(&mut story, &mut rng);
        }

        assert!(story.mutations > 0);
        assert_ne!(story.text, "The colony was founded.");
    }
}
