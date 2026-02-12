//! Demo of the Miasma Grid (Experimental).
//!
//! # Running this example
//!
//! This example requires the `nova` feature to be enabled.
//!
//! ```bash
//! cargo run --example miasma_test --features nova
//! ```

#[cfg(feature = "nova")]
use scale::experimental::miasma::MiasmaGrid;

fn main() {
    #[cfg(not(feature = "nova"))]
    {
        println!("❌ Feature 'nova' is NOT enabled.");
        println!("Please run with: cargo run --example miasma_test --features nova");
    }

    #[cfg(feature = "nova")]
    {
        let grid = MiasmaGrid::new(10, 10);
        println!(
            "Miasma grid created with size {}x{}",
            grid.width, grid.height
        );
    }
}
