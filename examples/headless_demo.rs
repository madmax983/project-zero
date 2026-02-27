//! Headless simulation example.
//!
//! Demonstrates how to run the simulation without a window or GPU context.

use scale::setup::{setup_world_with_config, SetupConfig};
use scale::shared::time::SimulationTime;
use scale::simulation::run_simulation_tick;

fn main() {
    println!("Initializing headless simulation...");

    // 1. Setup the world with headless configuration
    let config = SetupConfig {
        headless: true,
        ..Default::default()
    };
    let mut world = setup_world_with_config(config);

    println!("Simulation started (Headless Mode)");

    // 2. Run a few ticks
    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    // 3. Inspect state
    let time = world.resource::<SimulationTime>();
    println!("Current Tick: {}", time.tick);
}
