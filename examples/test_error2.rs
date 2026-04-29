use comfy_table::{presets::UTF8_FULL, Cell, Color as TableColor, Table};
use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut generator = NarrativeGenerator::default();
    let result = generator.load_from_files("./non_existent_folder");

    match result {
        Ok(_) => println!("Loaded successfully."),
        Err(e) => {
            let error_msg = format!("\n  {} \n", e);
            let action_msg = "  Action Required: Check Lore Directory.\n  Verify the folder path exists and contains markdown files. ";
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec![comfy_table::Cell::new(" ✗ LORE LOADING ERROR ")
                .add_attribute(comfy_table::Attribute::Bold)
                .fg(TableColor::Red)
                .bg(TableColor::DarkGrey)]);
            table.add_row(vec![Cell::new(&error_msg).fg(TableColor::White)]);
            table.add_row(vec![Cell::new(action_msg).fg(TableColor::Yellow)]);
            println!("{table}");
        }
    }
    Ok(())
}
