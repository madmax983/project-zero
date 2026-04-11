use scale::prelude::*;
use crossterm::style::{Color, Stylize};

fn main() -> anyhow::Result<()> {
    let generator = NarrativeGenerator::from_embedded();
    let mut context = NarrativeContext::default();
    context.insert("CIV_NAME", "Terran Dominion");

    let result = generator.generate("NON_EXISTENT_TEMPLATE", &context);

    match result {
        Ok(story) => println!("{}", story),
        Err(e) => {
            let error_msg = format!("{}", e);
            let width = error_msg.chars().count() + 4; // 2 for padding, 2 for icon
            let border = "─".repeat(width);
            let top_border = format!("╭{}╮", border);
            let bottom_border = format!("╰{}╯", border);

            println!("{}", top_border.with(Color::Cyan));
            println!(
                "│ {} {} │",
                "✗".with(Color::Red).bold(),
                error_msg.with(Color::White)
            );
            println!("{}", bottom_border.with(Color::Cyan));
        }
    }

    Ok(())
}
