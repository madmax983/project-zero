//! Demo of the NarrativeGenerator system.
//!
//! This example shows how to load procedural lore from files and generate
//! an event string using a template and context.

use scale::shared::narrative::{NarrativeGenerator, NarrativeContext};

fn main() -> anyhow::Result<()> {
    println!("🗣️  Echo: Nova Story Feature Demo");
    println!("=================================");

    // 1. Initialize Generator
    let mut generator = NarrativeGenerator::default();

    // 2. Load Lore Data
    // We assume the user is running from repo root
    println!("Loading lore from ./lore/ ...");
    generator.load_from_files("./lore")?;
    println!("Loaded {} templates and {} fragment types.",
        generator.template_count(),
        generator.fragment_count()
    );

    // 3. Prepare Context for a Story
    // We want to generate a CIVILIZATION_RISE event
    let mut context = NarrativeContext::default();
    context.insert("CIV_NAME", "Terran Dominion");
    context.insert("ORIGIN_STAR", "Sol");
    context.insert("YEAR", "2150");

    // Resolve EPITHET using CIV_EPITHET fragment manually because the names don't match
    if let Some(epithet) = generator.get_random_fragment("CIV_EPITHET") {
        context.insert("EPITHET", epithet);
    }

    // 4. Generate Story
    println!("\nGenerating CIVILIZATION_RISE event...");
    let story = generator.generate("CIVILIZATION_RISE", &context)?;

    println!("\n📜 Result:\n\"{}\"", story);

    Ok(())
}
