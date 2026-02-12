use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use scale::layer1::factions::{FactionId, FactionMember, Factions};
use scale::layer1::integration::faction_satisfaction_morale_bridge;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;

#[test]
fn test_faction_satisfaction_affects_morale() {
    let mut world = World::new();
    world.insert_resource(Factions::default());

    // Initialize factions
    {
        let mut factions = world.resource_mut::<Factions>();
        factions.initialize();

        // Set satisfaction to 0.5 (below 0.9 threshold)
        if let Some(data) = factions.map.get_mut(&FactionId::MinersGuild) {
            data.satisfaction = 0.5;
        }
    }

    // Spawn a pop in that faction
    let pop = world
        .spawn((
            Pop,
            Needs {
                leisure: 0.8,
                ..Default::default()
            },
            FactionMember {
                faction_id: Some(FactionId::MinersGuild),
            },
        ))
        .id();

    // Run system
    world
        .run_system_once(faction_satisfaction_morale_bridge)
        .unwrap();

    let needs = world.get::<Needs>(pop).unwrap();
    // Penalty = (0.9 - 0.5) * 0.001 = 0.4 * 0.001 = 0.0004
    // Expected = 0.8 - 0.0004 = 0.7996
    // Using EPSILON for float comparison
    let expected = 0.8 - ((0.9 - 0.5) * 0.001);
    assert!(
        (needs.leisure - expected).abs() < f32::EPSILON,
        "Leisure should drop due to faction dissatisfaction. Expected {}, Got {}",
        expected,
        needs.leisure
    );
}

#[test]
fn test_high_satisfaction_no_penalty() {
    let mut world = World::new();
    world.insert_resource(Factions::default());

    // Initialize factions
    {
        let mut factions = world.resource_mut::<Factions>();
        factions.initialize();

        // Set satisfaction to 1.0 (above 0.9 threshold)
        if let Some(data) = factions.map.get_mut(&FactionId::MinersGuild) {
            data.satisfaction = 1.0;
        }
    }

    // Spawn a pop
    let pop = world
        .spawn((
            Pop,
            Needs {
                leisure: 0.8,
                ..Default::default()
            },
            FactionMember {
                faction_id: Some(FactionId::MinersGuild),
            },
        ))
        .id();

    world
        .run_system_once(faction_satisfaction_morale_bridge)
        .unwrap();

    let needs = world.get::<Needs>(pop).unwrap();
    assert!(
        (needs.leisure - 0.8).abs() < f32::EPSILON,
        "Leisure should not change if satisfaction is high"
    );
}
