use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    // 1. Initialize Generator (loads embedded lore by default)
    let generator = NarrativeGenerator::from_embedded();

    // Or load from a directory (must contain TEMPLATES.md and FRAGMENTS.md)
    // let mut generator = NarrativeGenerator::default();
    // generator.load_from_files("./lore")?;

    // 2. Prepare Context
    let mut context = NarrativeContext::default();
    context.insert("CIV_NAME", "Terran Dominion");
    // Note: If required context variables are missing, `generate` will return an error.
    context.insert("ORIGIN_STAR", "Sol Prime");
    context.insert("YEAR", "2150");
    context.insert("CIV_EPITHET", "The First Ones");

    // 3. Generate Story
    let story = generator.generate("CIVILIZATION_RISE", &context)?;
    println!("{}", story);

    Ok(())
}
