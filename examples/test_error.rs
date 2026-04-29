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
            let error_msg = format!("\n  {} \n", e);
            let action_msg = "  Action Required: Check Template ID or Context.\n  Verify the name exists in your TEMPLATES.md. ";
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec![comfy_table::Cell::new(" ✗ NARRATIVE GENERATOR ERROR ")
                .add_attribute(comfy_table::Attribute::Bold)
                .fg(TableColor::Red)
                .bg(TableColor::DarkGrey)]);
            table.add_row(vec![Cell::new(&error_msg).fg(TableColor::White)]);
            table.add_row(vec![Cell::new(&action_msg).fg(TableColor::Yellow)]);
            println!("{table}");
        }
    }

    Ok(())
}
