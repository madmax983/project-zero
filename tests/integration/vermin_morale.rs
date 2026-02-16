use scale::layer1::memory::{Memories, MemoryType};
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;
use scale::layer1::vermin::VerminState;
use scale::setup::setup_world;

#[test]
fn test_vermin_affects_morale() {
    // 1. Setup World
    let mut world = setup_world();

    // Ensure VerminState is initialized
    if world.get_resource::<VerminState>().is_none() {
        world.insert_resource(VerminState::default());
    }

    // Set high severity
    {
        let mut vermin = world.resource_mut::<VerminState>();
        vermin.severity = 80.0;
    }

    // Set pressure to avoid suffocation
    if let Some(mut pressure) = world.get_resource_mut::<scale::layer1::pressure::PressureGrid>() {
        pressure.fill(1.0);
    }

    // 2. Spawn Pop
    let pop = world
        .spawn((
            Pop,
            Memories::default(),
            Needs::default(),
            scale::layer1::Health::default(),
            scale::layer1::GridPosition { x: 5, y: 5 },
        ))
        .id();

    // 3. Run Simulation Ticks
    // Run for enough ticks to ensure the probabilistic event occurs.
    // At 80 severity, chance is ~6% per tick.
    // Over 200 ticks, failure chance is negligible (~0.0004%).
    for _ in 0..500 {
        // Refill pressure to prevent suffocation (since edges leak to vacuum)
        if let Some(mut pressure) = world.get_resource_mut::<scale::layer1::pressure::PressureGrid>() {
            pressure.fill(1.0);
        }
        scale::simulation::run_simulation_tick(&mut world);
    }

    // 4. Assert Memory Gained
    let memories = world
        .get::<Memories>(pop)
        .expect("Pop should have Memories");

    let has_memory = memories
        .items
        .iter()
        .any(|m| m.memory_type == MemoryType::DisgustedByVermin);

    assert!(
        has_memory,
        "Pop should have acquired DisgustedByVermin memory due to high severity"
    );
}
