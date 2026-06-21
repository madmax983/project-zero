use crate::layer1::core::chronicle::Chronicle;
use bevy_ecs::prelude::Resource;

#[derive(Debug)]
pub struct Story {
    pub text: String,
    pub historical_date: u64,
    pub mutations: u32,
    pub genre: StoryGenre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryGenre {
    Heroic,
    Tragedy,
    Cautionary,
    Trivial,
}

#[derive(Debug, Default, Resource)]
pub struct OralTradition {
    pub stories: Vec<Story>,
}

impl OralTradition {
    pub fn add_story(&mut self, story: Story) {
        self.stories.push(story);
        bevy::log::warn_once!("Oral Tradition is disabled. Enable feature 'nova'.");
    }

    pub fn process_chronicles(&mut self, _chronicle: &Chronicle) {
        bevy::log::warn_once!("Oral Tradition is disabled. Enable feature 'nova'.");
    }
}
