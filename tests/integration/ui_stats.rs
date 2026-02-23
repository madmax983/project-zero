#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::{
        BuildMode, ColonyResources, DesignationMode, NamedLocations, Pop, Viewport,
    };
    use scale::shared::state::GameState;
    use scale::shared::time::{SimSpeed, SimulationTime};
    use scale::ui::status::get_status_string;

    #[test]
    fn test_status_bar_displays_stats() {
        // 1. Setup World
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 1000,
            speed: SimSpeed::Normal,
        });
        world.insert_resource(GameState::Running);
        world.insert_resource(BuildMode::default());
        world.insert_resource(DesignationMode::default());
        world.insert_resource(Viewport::default());
        world.insert_resource(NamedLocations::default());

        // 2. Add Data (Seam Input)
        let mut resources = ColonyResources::default();
        resources.food = 42.0;
        world.insert_resource(resources);

        world.spawn(Pop);
        world.spawn(Pop);
        world.spawn(Pop); // 3 Pops

        // 3. Simulate UI Logic (The Seam)
        // Extract data as render_status_bar does
        let pop_count = world.query::<&Pop>().iter(&world).count();
        let food = world.resource::<ColonyResources>().food;

        let status = get_status_string(
            1000,
            SimSpeed::Normal,
            false,
            &BuildMode::default(),
            &DesignationMode::default(),
            None,
            pop_count,
            food,
            0.0, // Rations
            0.0, // Tools
            0.8,
            1.0,  // Efficiency
            None, // Season
            None, // Solar Cycle
        );

        // 4. Verify Glue
        assert!(
            status.contains("Souls: 3"),
            "Status bar missing population count"
        );
        assert!(status.contains("Food: 42"), "Status bar missing food yield");
        assert!(status.contains("Morale: 80%"), "Status bar missing Morale");
        assert!(status.contains("Day 1000"), "Status bar missing Day");
    }
}
