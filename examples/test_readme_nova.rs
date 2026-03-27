// In Cargo.toml:
// scale = { version = "...", features = ["nova"] }

use scale::prelude::*;

fn main() {
    let mut tradition = OralTradition::default();

    // Add a story directly to the tradition
    let story = Story {
        text: "The colony survived the Great Frost.".to_string(),
        historical_date: 100,
        mutations: 0,
        genre: StoryGenre::Heroic,
    };
    tradition.add_story(story);

    // Inspect
    println!("{:?}", tradition.stories);
}
