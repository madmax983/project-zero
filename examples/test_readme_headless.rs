use scale::prelude::*;

fn main() {
    // 1. Setup the world with headless configuration
    let config = SetupConfig {
        headless: true,

    };
    let mut world = setup_world_with_config(config);

    // 2. Run a few ticks
    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    // 3. Inspect state
    let time = world.resource::<SimulationTime>();
    println!("Current Tick: {}", time.tick);
}
