//! Nova README example
// This file tests the Oral Tradition example snippet in the README.

fn main() {
    use scale::prelude::*;

    let mut tradition = OralTradition::default();

    let story = Story {
        text: "The colony survived the Great Frost.".to_string(),
        historical_date: 100,
        mutations: 0,
        genre: StoryGenre::Heroic,
    };
    tradition.add_story(story);

    // Inspect
    println!("{}", tradition);
}
