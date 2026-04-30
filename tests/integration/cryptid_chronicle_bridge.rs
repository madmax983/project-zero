use bevy::prelude::*;
use scale::layer1::anomalies::cryptid::{Cryptid, TraceItem, TraceKind, PopMood, VisionRadius, cryptid_trace_system, cryptid_observation_system};
use scale::layer1::map::GridPosition;
use scale::layer1::entities::pop::Pop;

#[test]
fn test_cryptid_trace_and_observation_chain() {
    let mut app = App::new();
    app.init_resource::<Time>();

    app.add_systems(Update, (
        cryptid_trace_system,
        cryptid_observation_system,
    ).chain());

    let _cryptid = app.world_mut().spawn((
        Cryptid { trace_timer: Timer::from_seconds(5.0, TimerMode::Repeating) },
        GridPosition { x: 5, y: 5 },
    )).id();

    let pop = app.world_mut().spawn((
        Pop,
        GridPosition { x: 6, y: 5 },
        PopMood { awe: 0.0, dread: 0.0 },
        VisionRadius(5.0),
    )).id();

    app.update();

    let mut time = app.world_mut().resource_mut::<Time>();
    time.advance_by(std::time::Duration::from_secs(6));
    app.update();

    let mood = app.world().get::<PopMood>(pop).unwrap();
    assert!(mood.awe > 0.0, "Pop should feel awe from cryptid");

    let mut query = app.world_mut().query::<(&TraceItem, &GridPosition)>();
    let traces: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(traces.len(), 1, "Cryptid should spawn trace");
    assert_eq!(traces[0].0.kind, TraceKind::Slime);
}
