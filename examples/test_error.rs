use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let generator = NarrativeGenerator::from_embedded();
    let mut context = NarrativeContext::default();
    context.insert("CIV_NAME", "Terran Dominion");

    let story = generator.generate("NON_EXISTENT_TEMPLATE", &context)?;
    println!("{}", story);

    Ok(())
}
