#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::map::GridPosition;
    use scale::layer1::orbital_crossfire::ImpactSite;
    use scale::layer2::debris::OrbitalDebris;
    use scale::layer2::generation::{ColonyLocation, Planet};
    use scale::setup::setup_world;
    use scale::simulation::build_simulation_schedule;

    #[test]
    fn test_debris_causes_orbital_impacts() {
        // 1. Setup World
        let mut world = setup_world();

        // 2. Find Colony Planet and set high debris
        let (planet_entity, _) = world
            .query::<(Entity, &ColonyLocation)>()
            .single(&world);

        // Ensure it has OrbitalDebris (added in previous step)
        let mut debris = world.get_mut::<OrbitalDebris>(planet_entity).expect("Colony planet should have OrbitalDebris");
        debris.amount = 100.0; // Very high amount to guarantee impact

        // 3. Register Systems
        // We use the full simulation schedule which SHOULD eventually include our bridge system
        let mut schedule = build_simulation_schedule();

        // 4. Run Schedule
        // Run multiple ticks to give chance for random event
        let mut event_occurred = false;
        for _ in 0..10 {
            schedule.run(&mut world);

            // Check for ImpactSite (persistent result of OrbitalEvent being processed)
            let sites = world.query::<&ImpactSite>().iter(&world).count();
            if sites > 0 {
                event_occurred = true;
                break;
            }
        }

        // 5. Assert
        assert!(event_occurred, "High orbital debris should trigger ImpactSite in Layer 1");
    }
}
