import sys

with open("src/layer1/oral_tradition.rs", "r") as f:
    content = f.read()

display_impl = """
#[cfg(feature = "nova")]
impl std::fmt::Display for OralTradition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use comfy_table::{presets::UTF8_FULL, Cell, Color as TableColor, Table};
        use crossterm::style::{Color, Stylize};

        writeln!(
            f,
            "{}",
            "╭── Oral Tradition (Stories) ───────────────────╮".with(Color::Cyan)
        )?;

        if self.stories.is_empty() {
            let text = format!("{:<47}", "No stories currently circulating.");
            writeln!(f, "│ {} │", text.with(Color::DarkGrey))?;
            writeln!(
                f,
                "{}",
                "╰───────────────────────────────────────────────╯".with(Color::Cyan)
            )?;
            return Ok(());
        } else {
            let text = format!(
                "{:<47}",
                format!("{} stories circulating.", self.stories.len())
            );
            writeln!(f, "│ {} │", text.with(Color::White))?;
            writeln!(
                f,
                "{}",
                "╰───────────────────────────────────────────────╯".with(Color::Cyan)
            )?;
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Genre").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Historical Date").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Mutations").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Story Text").add_attribute(comfy_table::Attribute::Bold),
            ]);

        for story in &self.stories {
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

        write!(f, "{}", table)
    }
}
"""

with open("src/layer1/oral_tradition.rs", "w") as f:
    f.write(content + "\n" + display_impl)
