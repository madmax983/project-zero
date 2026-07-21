use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let generator = NarrativeGenerator::from_embedded();
    let mut context = NarrativeContext::default();
    context.insert("CIV_NAME", "Terran Dominion");

    let result = generator.generate("NON_EXISTENT_TEMPLATE", &context);

    match result {
        Ok(story) => println!("{}", story),
        Err(e) => {

            println!("{}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
