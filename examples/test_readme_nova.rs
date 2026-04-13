//! Nova README example
// This file tests the Oral Tradition example snippet in the README.

fn main() {
    #[cfg(feature = "nova")]
    {
        use comfy_table::presets::UTF8_FULL;
        use comfy_table::{Cell, Color as TableColor, Table};
        use crossterm::style::{Color, Stylize};
        use scale::layer1::oral_tradition::{OralTradition, Story, StoryGenre};

        let mut tradition = OralTradition::default();

        let story = Story {
            text: "The colony survived the Great Frost.".to_string(),
            historical_date: 100,
            mutations: 0,
            genre: StoryGenre::Heroic,
        };
        tradition.add_story(story);

        println!(
            "\n{}",
            "╭── Oral Tradition (Stories) ───────────────────╮".with(Color::Cyan)
        );
        let text = format!(
            "{:<45}",
            format!("{} stories circulating.", tradition.stories.len())
        );
        println!("│ {} │", text.with(Color::White));
        println!(
            "{}",
            "╰───────────────────────────────────────────────╯".with(Color::Cyan)
        );

        let mut table = Table::new();
        table.load_preset(UTF8_FULL).set_header(vec![
            "Genre",
            "Historical Date",
            "Mutations",
            "Story Text",
        ]);

        for story in &tradition.stories {
            let genre_str = format!("{:?}", story.genre);
            let genre_color = match genre_str.as_str() {
                "Heroic" => TableColor::Yellow,
                "Tragedy" => TableColor::Red,
                "Cautionary" => TableColor::Magenta,
                _ => TableColor::DarkGrey,
            };

            table.add_row(vec![
                Cell::new(&genre_str).fg(genre_color),
                Cell::new(story.historical_date.to_string()).fg(TableColor::Cyan),
                Cell::new(story.mutations.to_string()).fg(TableColor::Cyan),
                Cell::new(&story.text).fg(TableColor::White),
            ]);
        }
        println!("{table}");
    }
}
