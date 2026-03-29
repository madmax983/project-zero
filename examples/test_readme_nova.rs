// This file tests the Oral Tradition example snippet in the README.

fn main() {
    #[cfg(feature = "nova")]
    {
        use scale::layer1::oral_tradition::{OralTradition, Story, StoryGenre};
        let mut tradition = OralTradition::default();

        let story = Story {
            text: "The colony survived the Great Frost.".to_string(),
            historical_date: 100,
            mutations: 0,
            genre: StoryGenre::Heroic,
        };
        tradition.add_story(story);
        println!("{:?}", tradition.stories);
    }
}
