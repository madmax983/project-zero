use bevy_ecs::prelude::*;
use scale::layer1::black_market::ColonyStats;
use scale::layer1::integration::update_unmet_luxury_system;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;

#[test]
fn test_unmet_luxury_updates_colony_stats() {
    let mut world = World::new();

    // Init ColonyStats
    world.insert_resource(ColonyStats::default());

    // Spawn 2 pops with high luxury need (meaning they lack luxury/leisure)
    world.spawn((
        Pop,
        Needs {
            leisure: 10.0,
            ..Default::default()
        },
    ));
    world.spawn((
        Pop,
        Needs {
            leisure: 20.0,
            ..Default::default()
        },
    ));

    // Spawn 1 pop with satisfied leisure
    world.spawn((
        Pop,
        Needs {
            leisure: 80.0,
            ..Default::default()
        },
    ));

    // Run the integration system
    let mut schedule = Schedule::default();
    schedule.add_systems(update_unmet_luxury_system);
    schedule.run(&mut world);

    // Assert that the colony stats resource was updated to count the 2 deprived pops
    let stats = world.resource::<ColonyStats>();
    assert_eq!(
        stats.unmet_luxury, 2,
        "Should count 2 pops with unmet luxury needs"
    );
}
