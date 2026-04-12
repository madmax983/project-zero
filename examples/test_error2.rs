use crossterm::style::{Color, Stylize};
use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut generator = NarrativeGenerator::default();
    let result = generator.load_from_files("./non_existent_folder");

    match result {
        Ok(_) => println!("Loaded successfully."),
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
