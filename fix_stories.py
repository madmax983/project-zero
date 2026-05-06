with open("src/bin/headless.rs", "r") as f:
    code = f.read()

search = """        let mut story_cell;

        if story.mutations > 0 {
            // When rendering in terminal, mixing embedded ANSI with comfy-table fg causes resets
            // which clears all colors. To fix this, we manually apply the base color to all non-mutated parts.
            let ct_color = match story.genre {
                StoryGenre::Heroic => crossterm::style::Color::Yellow,
                StoryGenre::Tragedy => crossterm::style::Color::Red,
                StoryGenre::Cautionary => crossterm::style::Color::Cyan,
                StoryGenre::Trivial => crossterm::style::Color::Grey,
            };"""

replace = """        let mut story_cell;

        if story.mutations > 0 {
            // When rendering in terminal, mixing embedded ANSI with comfy-table fg causes resets
            // which clears all colors. To fix this, we manually apply the base color to all non-mutated parts.
            let ct_color = match story.genre {
                StoryGenre::Heroic => crossterm::style::Color::Yellow,
                StoryGenre::Tragedy => crossterm::style::Color::Red,
                StoryGenre::Cautionary => crossterm::style::Color::Cyan,
                StoryGenre::Trivial => crossterm::style::Color::Grey,
            };"""

# Wait, `print_stories` already formats the mutations correctly! They are colored magenta and bold:
# `final_text.push_str(&m.magenta().bold().to_string());`
# Let's add icons to the genre instead.

search_genre = """        let genre_color = match story.genre {
            StoryGenre::Heroic => Color::Yellow,
            StoryGenre::Tragedy => Color::Red,
            StoryGenre::Cautionary => Color::Cyan,
            StoryGenre::Trivial => Color::Grey,
        };"""

replace_genre = """        let genre_color = match story.genre {
            StoryGenre::Heroic => Color::Yellow,
            StoryGenre::Tragedy => Color::Red,
            StoryGenre::Cautionary => Color::Cyan,
            StoryGenre::Trivial => Color::Grey,
        };

        let genre_text = match story.genre {
            StoryGenre::Heroic => "🌟 Heroic",
            StoryGenre::Tragedy => "🎭 Tragedy",
            StoryGenre::Cautionary => "⚠️ Cautionary",
            StoryGenre::Trivial => "📝 Trivial",
        };"""

search_row = """        table.add_row(vec![
            Cell::new(story.historical_date.to_string()),
            Cell::new(format!("{:?}", story.genre)).fg(genre_color),
            Cell::new(story.mutations.to_string()),
            story_cell,
        ]);"""

replace_row = """        table.add_row(vec![
            Cell::new(story.historical_date.to_string()),
            Cell::new(genre_text).fg(genre_color),
            Cell::new(story.mutations.to_string()),
            story_cell,
        ]);"""

code = code.replace(search_genre, replace_genre)
code = code.replace(search_row, replace_row)

with open("src/bin/headless.rs", "w") as f:
    f.write(code)
