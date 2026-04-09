//! Headless simulation example.
//!
//! Demonstrates how to run the simulation without a window or GPU context.

use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Color as TableColor, Table};
use crossterm::style::{Color, Stylize};
use scale::layer1::pop::Pop;
use scale::layer1::resources::ColonyResources;
use scale::setup::{setup_world_with_config, SetupConfig};
use scale::shared::time::SimulationTime;
use scale::simulation::run_simulation_tick;
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

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Category", "Metric", "Value"]);

    table.add_row(vec![
        Cell::new("Population").fg(TableColor::Cyan),
        Cell::new("Citizens"),
        Cell::new(pop_count.to_string()),
    ]);

    table.add_row(vec![
        Cell::new("Basic").fg(TableColor::Yellow),
        Cell::new("Food"),
        Cell::new(format!("{:.1}", resources.food)).fg(if resources.food < 20.0 {
            TableColor::Red
        } else {
            TableColor::Green
        }),
    ]);

    table.add_row(vec![
        Cell::new(""),
        Cell::new("Wood"),
        Cell::new(format!("{:.1}", resources.wood)),
    ]);

    table.add_row(vec![
        Cell::new(""),
        Cell::new("Stone"),
        Cell::new(format!("{:.1}", resources.stone)),
    ]);

    table.add_row(vec![
        Cell::new(""),
        Cell::new("Water"),
        Cell::new(format!("{:.1}", resources.water)).fg(TableColor::Blue),
    ]);

    table.add_row(vec![
        Cell::new("Advanced").fg(TableColor::Magenta),
        Cell::new("Knowledge"),
        Cell::new(format!("{:.1}", resources.knowledge)).fg(TableColor::Cyan),
    ]);

    // Align the Value column to the right for better readability
    if let Some(col) = table.column_mut(2) {
        col.set_cell_alignment(CellAlignment::Right);
    }

    println!("\n{table}");
}
