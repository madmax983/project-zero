//! Minimal example of the Oral Tradition (Nova) feature.
//!
//! Demonstrates how to initialize the system, feed it historical events,
//! and process them into legends.
//!
//! Run with: `cargo run --features nova --example minimal_nova_demo`

#[cfg(feature = "nova")]
mod demo {
    use bevy_ecs::prelude::*;
    use crossterm::style::{Color, Stylize};
    use scale::layer1::chronicle::{Chronicle, EventImportance};
    use scale::layer1::oral_tradition::{collect_chronicles_system, OralTradition};

    pub fn run() {
        let mut world = World::new();

        // 1. Initialize Resources
        world.insert_resource(OralTradition::default());
        world.insert_resource(Chronicle::default());

        println!("{}", "╭── Initial State ──────────────────────────────╮".with(Color::Cyan));
        let tradition = world.resource::<OralTradition>();
        if tradition.stories.is_empty() {
            println!("│ {} │", "No stories currently circulating.".with(Color::DarkGrey));
        } else {
            println!("│ {} │", format!("{} stories circulating.", tradition.stories.len()).with(Color::White));
        }
        println!("{}", "╰───────────────────────────────────────────────╯".with(Color::Cyan));

        // 2. Add a historical event
        println!("\n{}", "╭── Adding Event ───────────────────────────────╮".with(Color::Cyan));
        world.resource_mut::<Chronicle>().add_event(
            100, // tick
            "The colony survived the Great Frost.".to_string(),
            EventImportance::Legendary,
        );
        println!("│ {} │", "✓ Event successfully added to Chronicle.".with(Color::Green));
        println!("{}", "╰───────────────────────────────────────────────╯".with(Color::Cyan));

        // 3. Run the system to process events into stories
        println!("\n{}", "╭── Processing Events ──────────────────────────╮".with(Color::Cyan));
        let mut schedule = Schedule::default();
        schedule.add_systems(collect_chronicles_system);
        schedule.run(&mut world);
        println!("│ {} │", "✓ Events processed into Oral Tradition.".with(Color::Green));
        println!("{}", "╰───────────────────────────────────────────────╯".with(Color::Cyan));

        // 4. Inspect the result
        println!("\n{}", "╭── Updated Oral Tradition ─────────────────────╮".with(Color::Cyan));
        let tradition = world.resource::<OralTradition>();
        for story in &tradition.stories {
            let genre_str = format!("{:?}", story.genre);
            let genre_styled = match genre_str.as_str() {
                "Heroic" => genre_str.with(Color::Yellow).bold(),
                "Tragedy" => genre_str.with(Color::Red).bold(),
                "Cautionary" => genre_str.with(Color::Magenta).bold(),
                _ => genre_str.with(Color::DarkGrey).bold(),
            };

            println!("│ • [{}] {}", genre_styled, story.text.clone().with(Color::White));
            println!("│   Origin Tick: {} | Mutations: {}",
                     story.origin_tick.to_string().with(Color::Cyan),
                     story.mutations.to_string().with(Color::Cyan));
            println!("│");
        }
        println!("{}", "╰───────────────────────────────────────────────╯".with(Color::Cyan));
    }
}

fn main() {
    #[cfg(feature = "nova")]
    demo::run();

    #[cfg(not(feature = "nova"))]
    println!(
        "This example requires the 'nova' feature.\nRun with: cargo run --features nova --example minimal_nova_demo"
    );
}
