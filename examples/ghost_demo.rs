//! Demo of the experimental Ghosts feature.
//!
//! # Running this example
//!
//! This example requires the `nova` feature to be enabled.
//!
//! ```bash
//! cargo run --example ghost_demo --features nova
//! ```

#[cfg(feature = "nova")]
use scale::experimental::ghosts::{Ghost, Ectoplasm};

fn main() -> anyhow::Result<()> {
    println!("👻 Ghost Demo");
    println!("===========");

    #[cfg(not(feature = "nova"))]
    {
        println!("❌ Feature 'nova' is NOT enabled.");
        println!("Please run with: cargo run --example ghost_demo --features nova");
        // We don't return error so CI doesn't fail if checking all examples
    }

    #[cfg(feature = "nova")]
    {
        println!("✅ Feature 'nova' IS enabled.");

        // Demonstrate usage
        let ghost = Ghost;
        let ectoplasm = Ectoplasm::default();

        println!("Spawned a {:?} with {:?}!", ghost, ectoplasm);
        println!("(Note: In the full game, ghosts spawn when colonists die)");
    }

    Ok(())
}
