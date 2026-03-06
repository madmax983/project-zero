//! Minimal example of the Oral Tradition (Nova) feature.
//!
//! Demonstrates how to initialize the system, feed it historical events,
//! and process them into legends.
//!
//! Run with: `cargo run --features nova --example minimal_nova_demo`

#[cfg(feature = "nova")]
mod demo {
    use bevy_ecs::prelude::*;
    use scale::layer1::chronicle::{Chronicle, EventImportance};
    use scale::layer1::oral_tradition::{collect_chronicles_system, OralTradition};

    pub fn run() {
        let mut world = World::new();

        // 1. Initialize Resources
        world.insert_resource(OralTradition::default());
        world.insert_resource(Chronicle::default());

        println!("--- Initial State ---");
        print!("{}", world.resource::<OralTradition>());

        // 2. Add a historical event
        println!("\n--- Adding Event ---");
        world.resource_mut::<Chronicle>().add_event(
            100, // tick
            "The colony survived the Great Frost.".to_string(),
            EventImportance::Legendary,
        );
        println!("Event added to Chronicle.");

        // 3. Run the system to process events into stories
        println!("\n--- Processing ---");
        let mut schedule = Schedule::default();
        schedule.add_systems(collect_chronicles_system);
        schedule.run(&mut world);

        // 4. Inspect the result
        let tradition = world.resource::<OralTradition>();
        println!("Oral Tradition updated:");
        print!("{tradition}");
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
