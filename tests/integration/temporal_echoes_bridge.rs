use bevy_ecs::prelude::*;
use scale::layer1::anomalies::temporal_echoes::{
    process_temporal_decay_system, process_temporal_echo_system, BuildingAge, ChronoAnomaly,
    TemporalDecayExperiencedEvent, TemporalEchoExperiencedEvent,
};
use scale::layer1::architecture::building::{Building, BuildingType};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::temporal_echo_chronicle_bridge;
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::psychology::stress::StressTracker;
use scale::layer1::skills::Skills;

fn setup_world() -> World {
    let mut world = World::new();
    world.init_resource::<Events<TemporalEchoExperiencedEvent>>();
    world.init_resource::<Events<TemporalDecayExperiencedEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();
    world
}

#[test]
fn test_temporal_echo_emits_chronicle_event() {
    let mut world = setup_world();
    let anomaly_pos = GridPosition { x: 50, y: 50 };
    world.spawn((ChronoAnomaly { radius: 2.0 }, anomaly_pos));

    world.spawn((
        Pop,
        GridPosition { x: 50, y: 50 },
        Skills { xp: std::collections::HashMap::new() },
        StressTracker { accumulated_stress: 0.0 },
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems((process_temporal_echo_system, temporal_echo_chronicle_bridge).chain());
    schedule.run(&mut world);

    let echo_events = world.resource::<Events<TemporalEchoExperiencedEvent>>();
    assert_eq!(echo_events.get_cursor().read(echo_events).count(), 1, "Should emit echo event");

    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    assert_eq!(chronicle_events.get_cursor().read(chronicle_events).count(), 1, "Should emit chronicle event");
}

#[test]
fn test_temporal_decay_emits_chronicle_event() {
    let mut world = setup_world();
    let anomaly_pos = GridPosition { x: 50, y: 50 };
    world.spawn((ChronoAnomaly { radius: 2.0 }, anomaly_pos));

    world.spawn((
        Building { building_type: BuildingType::Housing },
        GridPosition { x: 50, y: 50 },
        BuildingAge { ticks: 100 },
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems((process_temporal_decay_system, temporal_echo_chronicle_bridge).chain());
    schedule.run(&mut world);

    let decay_events = world.resource::<Events<TemporalDecayExperiencedEvent>>();
    assert_eq!(decay_events.get_cursor().read(decay_events).count(), 1, "Should emit decay event");

    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    assert_eq!(chronicle_events.get_cursor().read(chronicle_events).count(), 1, "Should emit chronicle event");
}