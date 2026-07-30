use scale::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut generator = NarrativeGenerator::default();
    let result = generator.load_from_files("./non_existent_folder");

    match result {
        Ok(_) => println!("Loaded successfully."),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
