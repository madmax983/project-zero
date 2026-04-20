use comfy_table::{presets::UTF8_FULL, Cell, Color as TableColor, Table};
use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut generator = NarrativeGenerator::default();
    let result = generator.load_from_files("./non_existent_folder");

    match result {
        Ok(_) => println!("Loaded successfully."),
        Err(e) => {
            let error_msg = format!(" {}", e);
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec![comfy_table::Cell::new("✗ ERROR")
                .add_attribute(comfy_table::Attribute::Bold)
                .fg(TableColor::Red)]);
            table.add_row(vec![Cell::new(&error_msg).fg(TableColor::DarkGrey)]);
            println!("{table}");
        }
    }
    Ok(())
}
