import re

with open('src/simulation.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''    #[test]
    fn test_schedule_runs_on_fresh_world() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        // Initialize Detection Risk for test
        world.init_resource::<crate::layer3::silence::DetectionRisk>();
        world.init_resource::<Events<crate::layer3::silence::HostileSpawnEvent>>();

        world.init_resource::<Events<crate::layer1::spiteful_will::InheritanceEvent>>();
        world.init_resource::<Events<crate::layer1::spiteful_will::OverrideWillEvent>>();
        world
            .init_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>();''',
'''    #[test]
    fn test_schedule_runs_on_fresh_world() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        // Initialize Detection Risk for test
        world.init_resource::<crate::layer3::silence::DetectionRisk>();
        world.init_resource::<Events<crate::layer3::silence::HostileSpawnEvent>>();

        world.init_resource::<Events<crate::layer1::spiteful_will::InheritanceEvent>>();
        world.init_resource::<Events<crate::layer1::spiteful_will::OverrideWillEvent>>();
        world
            .init_resource::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>();
        world.init_resource::<Events<crate::layer1::gravitational_debt::DebtReleaseEvent>>();'''
)

with open('src/simulation.rs', 'w') as f:
    f.write(content)
