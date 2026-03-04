import os

with open("src/simulation.rs", "r") as f:
    content = f.read()

content = content.replace(
    """    #[test]
    fn test_schedule_runs_on_fresh_world() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        let schedule = build_simulation_schedule();
        world.add_schedule(schedule);
        world.run_schedule(SimulationSchedule);

        // Should not panic — all systems run correctly on a fresh world
    }""",
    """    #[test]
    fn test_schedule_runs_on_fresh_world() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        // Initialize Detection Risk for test
        world.init_resource::<crate::layer3::silence::DetectionRisk>();

        let schedule = build_simulation_schedule();
        world.add_schedule(schedule);
        world.run_schedule(SimulationSchedule);

        // Should not panic — all systems run correctly on a fresh world
    }"""
)

with open("src/simulation.rs", "w") as f:
    f.write(content)
