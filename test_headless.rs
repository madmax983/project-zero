use scale::prelude::*;

fn main() {
    let config = SetupConfig {
        headless: true,
        ..Default::default()
    };
    let mut world = setup_world_with_config(config);

    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    let time = world.resource::<SimulationTime>();
    println!("Current Tick: {}", time.tick);
}
