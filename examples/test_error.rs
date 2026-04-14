use comfy_table::{presets::UTF8_FULL, Cell, Color as TableColor, Table};
use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let generator = NarrativeGenerator::from_embedded();
    let mut context = NarrativeContext::default();
    context.insert("CIV_NAME", "Terran Dominion");

    let result = generator.generate("NON_EXISTENT_TEMPLATE", &context);

    match result {
        Ok(story) => println!("{}", story),
        Err(e) => {
            let error_msg = format!("✗ {}", e);
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.add_row(vec![Cell::new(&error_msg).fg(TableColor::Red)]);
            println!("{table}");
        }
    }

    Ok(())
}
