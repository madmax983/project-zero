//! System for Oral Tradition and Folk Tales.
//!
//! Converts colony history (Chronicle) into living legends that are shared in taverns.
//! Stories evolve over time, gaining mutations and providing buffs to listeners.

use crate::layer1::core::chronicle::{Chronicle, EventImportance};
use crate::layer1::needs::Needs;
use crate::layer1::social::Tavern;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

/// A legend that has evolved from a historical event within the [`OralTradition`].
///
/// Legends begin as factual [`crate::layer1::core::chronicle::Chronicle`] entries but morph over time through
/// successive telling and retelling in taverns.
///
/// > ⚠️ **Nova Feature:** While this struct is always defined, its systems and effects are only active when the `nova` feature is enabled.
///
/// # Examples
///
/// ```
/// use scale::layer1::oral_tradition::{Story, StoryGenre};
///
/// let legend = Story {
///     text: "The colony survived the Great Frost.".to_string(),
///     historical_date: 100,
///     mutations: 0,
///     genre: StoryGenre::Heroic,
/// };
///
/// assert_eq!(legend.mutations, 0);
/// ```
#[derive(Debug, Clone)]
pub struct Story {
    /// The current text of the story.
    pub text: String,
    /// The historical date when the event happened.
    pub historical_date: u64,
    /// How many times the story has mutated.
    pub mutations: u32,
    /// The type/genre of the story.
    pub genre: StoryGenre,
}

/// The thematic genre of a [`Story`], which dictates how it affects those who hear it.
///
/// Different genres resonate differently with the populace:
/// - **Heroic** tales inspire and boost morale.
/// - **Cautionary** tales increase alertness.
/// - **Tragedies** foster empathy.
///
/// # Examples
///
/// ```
/// use scale::layer1::oral_tradition::StoryGenre;
///
/// let genre = StoryGenre::Cautionary;
/// assert_eq!(format!("{}", genre), "⚠️ Cautionary");
/// ```
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

pub const MAX_STORIES: usize = 100;

/// The collective repository of legends, myths, and rumors known to the colony.
///
/// `OralTradition` periodically scans the [`crate::layer1::core::chronicle::Chronicle`] for new events and seeds
/// them as factual stories. These stories are later shared in `Tavern`s, where they mutate.
///
/// > ⚠️ **Nova Feature:** While this resource is always defined, its active systems (like `collect_chronicles_system`) will emit warnings and do nothing unless the `nova` feature is enabled.
///
/// # Examples
///
/// ```
/// use scale::layer1::oral_tradition::{OralTradition, Story, StoryGenre};
///
/// let mut tradition = OralTradition::default();
/// tradition.add_story(Story {
///     text: "A strange object fell from the sky.".to_string(),
///     historical_date: 42,
///     mutations: 0,
///     genre: StoryGenre::Cautionary,
/// });
///
/// assert_eq!(tradition.stories.len(), 1);
/// ```
#[derive(Resource, Default, Debug)]
pub struct OralTradition {
    /// The collection of known stories.
    pub stories: Vec<Story>,
    /// The tick of the last processed chronicle event.
    pub last_processed_tick: u64,
}

impl OralTradition {
    /// Adds a [`Story`] to the tradition if a story from the same historical date
    /// does not already exist. If adding the story exceeds `MAX_STORIES`, the oldest
    /// story is forgotten.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::oral_tradition::{OralTradition, Story, StoryGenre};
    ///
    /// let mut tradition = OralTradition::default();
    /// tradition.add_story(Story {
    ///     text: "First landing.".to_string(),
    ///     historical_date: 0,
    ///     mutations: 0,
    ///     genre: StoryGenre::Trivial,
    /// });
    ///
    /// // Adding the same date again does nothing.
    /// tradition.add_story(Story {
    ///     text: "Another perspective on first landing.".to_string(),
    ///     historical_date: 0,
    ///     mutations: 0,
    ///     genre: StoryGenre::Trivial,
    /// });
    ///
    /// assert_eq!(tradition.stories.len(), 1);
    /// ```
    pub fn add_story(&mut self, story: Story) {
        if !self
            .stories
            .iter()
            .any(|s| s.historical_date == story.historical_date)
        {
            self.stories.push(story);
            if self.stories.len() > MAX_STORIES {
                self.stories.remove(0);
            }
        }
    }

    /// Evaluates new [`crate::layer1::core::chronicle::Chronicle`] events since the last processing tick and
    /// converts them into new `Story` entities based on keyword heuristics and importance.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::oral_tradition::{OralTradition, StoryGenre};
    /// use scale::layer1::core::chronicle::{Chronicle, EventImportance};
    ///
    /// let mut chronicle = Chronicle::default();
    /// chronicle.add_event(10, "A colonist died of starvation.".to_string(), EventImportance::Major);
    ///
    /// let mut tradition = OralTradition::default();
    /// tradition.process_chronicles(&chronicle);
    ///
    /// assert_eq!(tradition.stories[0].genre, StoryGenre::Tragedy);
    /// ```
    /// ⚡ Bolt Optimization:
    /// Removed intermediate `.collect::<Vec<_>>()` chain over filtered chronicle events.
    /// Processing `chronicle.events.iter()` directly prevents unnecessary O(N) memory
    /// allocations per simulation tick, significantly reducing memory pressure.
    pub fn process_chronicles(&mut self, chronicle: &Chronicle) {
        let mut has_new = false;
        let mut last_tick = self.last_processed_tick;
        let current_processed_tick = self.last_processed_tick;

        for event in chronicle
            .events
            .iter()
            .filter(|e| e.tick > current_processed_tick)
        {
            has_new = true;
            last_tick = event.tick;
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
                historical_date: event.tick,
                mutations: 0,
                genre,
            };

            self.add_story(story);
        }

        if has_new {
            self.last_processed_tick = last_tick;
        }
    }
}

/// Evaluates the [`crate::layer1::core::chronicle::Chronicle`] each tick and seeds the [`OralTradition`] with new events.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::oral_tradition::{OralTradition, collect_chronicles_system};
/// use scale::layer1::core::chronicle::Chronicle;
///
/// let mut world = World::new();
/// world.insert_resource(OralTradition::default());
/// world.insert_resource(Chronicle::default());
///
/// // Provide the event queue that the system queries
/// world.insert_resource(bevy_ecs::event::Events::<scale::layer1::core::chronicle::AddChronicleEvent>::default());
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(collect_chronicles_system);
/// schedule.run(&mut world);
/// ```
pub fn collect_chronicles_system(mut tradition: ResMut<OralTradition>, chronicle: Res<Chronicle>) {
    tradition.process_chronicles(&chronicle);
}

#[cfg(not(feature = "nova"))]
pub fn collect_chronicles_system() {
    bevy::log::warn_once!("The `nova` feature is not enabled! `collect_chronicles_system` will do nothing. Please add `features = [\"nova\"]` to your Cargo.toml.");
}

/// Facilitates the telling of tales within `Tavern`s.
///
/// When multiple pops gather in a tavern, there is a chance they share a story
/// from the [`OralTradition`]. Hearing a story affects the listeners' needs
/// according to the story's `StoryGenre`. Repeated tellings have a chance
/// to mutate the story text, turning history into myth.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::oral_tradition::{OralTradition, storytelling_system};
/// use scale::shared::log::MessageLog;
///
/// let mut world = World::new();
/// world.insert_resource(OralTradition::default());
/// world.insert_resource(MessageLog::default());
///
/// // Provide missing resources potentially needed by other systems in full integrations, though storytelling might just need what's here.
/// // storytelling_system queries mut tradition, tavern_query, pop_query, log. All are present or valid empty queries.
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(storytelling_system);
/// schedule.run(&mut world);
/// ```
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

#[cfg(not(feature = "nova"))]
pub fn storytelling_system() {
    bevy::log::warn_once!("The `nova` feature is not enabled! `storytelling_system` will do nothing. Please add `features = [\"nova\"]` to your Cargo.toml.");
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
        if let Some(suffix) = suffixes.choose(rng) {
            if !story.text.ends_with(suffix) {
                story.text.push_str(suffix);
            }
        }
    } else {
        // Replace word
        if let Some((target, replacement)) = replacements.choose(rng) {
            // Case insensitive replacement would be better but simple replace is fine for MVP
            story.text = story.text.replace(target, replacement);
        }
    }

    story.mutations += 1;
}

impl std::fmt::Display for StoryGenre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Heroic => write!(f, "🌟 Heroic"),
            Self::Tragedy => write!(f, "🎭 Tragedy"),
            Self::Cautionary => write!(f, "⚠️ Cautionary"),
            Self::Trivial => write!(f, "📜 Trivial"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::layer1::core::chronicle::{Chronicle, EventImportance};

    use crate::layer1::needs::Needs;
    use crate::layer1::social::Tavern;

    #[test]
    fn test_collect_chronicles() {
        let mut world = World::new();
        world.insert_resource(OralTradition::default());
        let mut chronicle = Chronicle::default();
        chronicle.add_event(100, "Heroic Deed".to_string(), EventImportance::Legendary);
        world.insert_resource(chronicle);

        let mut schedule = Schedule::default();
        schedule.add_systems(collect_chronicles_system);
        schedule.run(&mut world);

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

        let mut schedule = Schedule::default();
        schedule.add_systems(collect_chronicles_system);
        schedule.run(&mut world);

        let tradition = world.resource::<OralTradition>();
        assert_eq!(tradition.stories[0].genre, StoryGenre::Tragedy);
    }

    #[test]
    fn test_storytelling_buffs() {
        use crate::shared::log::MessageLog;
        let mut world = World::new();
        let mut tradition = OralTradition::default();
        tradition.add_story(Story {
            text: "Heroic Tale".to_string(),
            historical_date: 1,
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
            let mut schedule = Schedule::default();
            schedule.add_systems(storytelling_system);
            schedule.run(&mut world);
        }

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure > 0.5, "Heroic story should boost leisure");
    }

    #[test]
    fn test_mutation() {
        let mut rng = rand::thread_rng();
        let mut story = Story {
            text: "The colony was founded.".to_string(),
            historical_date: 0,
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
