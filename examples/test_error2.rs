use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut generator = NarrativeGenerator::default();
    generator.load_from_files("./non_existent_folder")?;
    Ok(())
}
