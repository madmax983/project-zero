//! Minimal example of the Oral Tradition (Nova) feature.
//!
//! Demonstrates how to initialize the system, feed it historical events,
//! and process them into legends.
//!
//! Run with: `cargo run --features nova --example minimal_nova_demo`

use crossterm::style::{Color, Stylize};
use scale::prelude::*;

fn main() {
    // 1. Initialize Resources
    let mut tradition = OralTradition::default();
    let mut chronicle = Chronicle::default();

    println!(
        "{}",
        "╭── Initial State ──────────────────────────────╮".with(Color::Cyan)
    );
    if tradition.stories.is_empty() {
        let text = format!("{:<47}", "No stories currently circulating.");
        println!("│ {} │", text.with(Color::DarkGrey));
    } else {
        let text = format!(
            "{:<47}",
            format!("{} stories circulating.", tradition.stories.len())
        );
        println!("│ {} │", text.with(Color::White));
    }
    println!(
        "{}",
        "╰───────────────────────────────────────────────╯".with(Color::Cyan)
    );

    // 2. Add a historical event
    println!(
        "\n{}",
        "╭── Adding Event ───────────────────────────────╮".with(Color::Cyan)
    );
    chronicle.add_event(
        100, // tick
        "The colony survived the Great Frost.".to_string(),
        EventImportance::Legendary,
    );
    let text = format!("{:<47}", "✓ Event successfully added to Chronicle.");
    println!("│ {} │", text.with(Color::Green));
    println!(
        "{}",
        "╰───────────────────────────────────────────────╯".with(Color::Cyan)
    );

    // 3. Run the system to process events into stories
    println!(
        "\n{}",
        "╭── Processing Events ──────────────────────────╮".with(Color::Cyan)
    );
    // Use the simplified API
    tradition.process_chronicles(&chronicle);
    let text = format!("{:<47}", "✓ Events processed into Oral Tradition.");
    println!("│ {} │", text.with(Color::Green));
    println!(
        "{}",
        "╰───────────────────────────────────────────────╯".with(Color::Cyan)
    );

    // 4. Inspect the result
    println!("\n{}", "Updated Oral Tradition".with(Color::Cyan).bold());

    println!("{}", tradition);
}
