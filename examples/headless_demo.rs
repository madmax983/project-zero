//! Headless simulation example.
//!
//! Demonstrates how to run the simulation without a window or GPU context.

use crossterm::style::{Color, Stylize};
use scale::layer1::resources::ColonyResources;
use scale::prelude::*;
use std::io::{self, Write};

fn main() {
    println!(
        "{} {}",
        "✓".with(Color::Green).bold(),
        "Initializing headless simulation..."
            .with(Color::Cyan)
            .bold()
    );

    // 1. Setup the world with headless configuration
    let config = SetupConfig {
        headless: true,
        ..Default::default()
    };
    let mut world = setup_world_with_config(config);

    println!(
        "{} {}",
        "✓".with(Color::Green).bold(),
        "Simulation started (Headless Mode)"
            .with(Color::Green)
            .bold()
    );

    // 2. Run a few ticks
    print!("{} ", "⠋".with(Color::Cyan).bold());
    for _ in 0..10 {
        print!("{}", ".".with(Color::DarkGrey));
        let _ = io::stdout().flush();
        run_simulation_tick(&mut world);
    }
    println!(" {}", "Done!".with(Color::Green).bold());

    // 3. Inspect state
    let pop_count = world.query::<&Pop>().iter(&world).count();

    let time = world.resource::<SimulationTime>();
    let resources = world.resource::<ColonyResources>();

    println!();
    println!(
        "{}",
        "╭── Simulation Dashboard ───────────────────────╮".with(Color::Cyan)
    );
    println!(
        "│ {} │",
        format!("Current Tick: {:<31}", time.tick).with(Color::White)
    );
    println!(
        "{}",
        "╰───────────────────────────────────────────────╯".with(Color::Cyan)
    );

    println!("\n[Category] | [Metric] | [Value]");
    println!("👥 Population | Citizens | {}", pop_count);
    println!("📦 Basic Resources | 🍖 Food | {:.1}", resources.food);
    println!("📦 Basic Resources | 🪵 Wood | {:.1}", resources.wood);
    println!("📦 Basic Resources | 🪨 Stone | {:.1}", resources.stone);
    println!("📦 Basic Resources | 💧 Water | {:.1}", resources.water);
    println!("🔬 Advanced | Knowledge | {:.1}", resources.knowledge);
}
