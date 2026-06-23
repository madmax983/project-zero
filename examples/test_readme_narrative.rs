//! Narrative README example
use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let generator = NarrativeGenerator::from_embedded();

    let mut context = NarrativeContext::default();
    context.insert("CIV_NAME", "Terran Dominion");
    context.insert("ORIGIN_STAR", "Sol Prime");
    context.insert("YEAR", "2150");
    context.insert("CIV_EPITHET", "The First Ones");

    match generator.generate("CIVILIZATION_RISE", &context) {
        Ok(story) => println!("{}", story),
        Err(e) => {
            let table = e.to_table();
            println!("{table}");
            std::process::exit(1);
        }
    }

    Ok(())
}
