//! Narrative README example
use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let generator = NarrativeGenerator::from_embedded();

    let mut context = NarrativeContext::default();
    context.insert("CIV_NAME", "Terran Dominion");
    context.insert("ORIGIN_STAR", "Sol Prime");
    context.insert("YEAR", "2150");
    context.insert("CIV_EPITHET", "The First Ones");

    let story = generator.generate("CIVILIZATION_RISE", &context)?;
    println!("{}", story);

    Ok(())
}
