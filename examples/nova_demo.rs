//! Demo of all Project Nova experimental features.
//!
//! # Running this example
//!
//! Run without `nova` features:
//! ```bash
//! cargo run --example nova_demo
//! ```
//!
//! Run WITH `nova` features:
//! ```bash
//! cargo run --example nova_demo --features nova
//! ```

use scale::experimental::biography::Biography;
use scale::experimental::dreams::Dream;
#[cfg(feature = "nova")]
use scale::experimental::ghosts::{Ectoplasm, Ghost};
#[cfg(feature = "nova")]
use scale::experimental::miasma::MiasmaGrid;

fn main() -> anyhow::Result<()> {
    println!("🌟 Project Nova Feature Demo");
    println!("===========================");

    // 1. Features available by default
    println!("\n[1] Default Experimental Features (Always Available):");

    // Dreams
    let dream = Dream {
        content: "dreamed of electric sheep".to_string(),
        tick: 100,
        impact: 0.1,
    };
    println!("  ✅ Dreams: Created a dream: '{}' (Impact: {})", dream.content, dream.impact);

    // Biography
    let mut bio = Biography::default();
    bio.add_event(1, "Joined the colony.".to_string());
    println!("  ✅ Biography: Created bio with {} event(s): {:?}", bio.events.len(), bio.events[0].text);

    // 2. Features requiring 'nova' flag
    println!("\n[2] Nova-Exclusive Features:");

    #[cfg(feature = "nova")]
    {
        // Ghosts
        let ghost = Ghost;
        let ectoplasm = Ectoplasm::default();
        println!("  ✅ Ghosts: Spawned a {:?} with {:?}!", ghost, ectoplasm);

        // Miasma
        let grid = MiasmaGrid::new(10, 10);
        println!("  ✅ Miasma: Initialized MiasmaGrid ({}x{}).", grid.width, grid.height);
    }

    #[cfg(not(feature = "nova"))]
    {
        println!("  ❌ Ghosts: NOT AVAILABLE (requires --features nova)");
        println!("  ❌ Miasma: NOT AVAILABLE (requires --features nova)");
        println!("\n💡 Tip: Run `cargo run --example nova_demo --features nova` to enable these features.");
    }

    Ok(())
}
