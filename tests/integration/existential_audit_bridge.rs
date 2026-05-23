use bevy::prelude::*;
use scale::layer1::economy::existential_audit::{existential_audit_system, existential_crisis_decay_system, PrecursorAI, IndustrialBuilding, ExistentialCrisis};
use scale::layer1::pop::Pop;
use scale::shared::time::SimulationTime;

#[test]
fn test_existential_audit_integration() {
    let mut app = App::new();
    app.init_resource::<SimulationTime>();
    app.insert_resource(PrecursorAI { next_audit_tick: 100 });

    app.add_systems(Update, (existential_audit_system, existential_crisis_decay_system).chain());

    let pop_entity = app.world_mut().spawn(Pop).id();
    app.world_mut().spawn(IndustrialBuilding { efficiency: 100.0, cultural_value: 0.0 });

    // Tick to 100 to trigger audit
    let mut time = app.world_mut().resource_mut::<SimulationTime>();
    time.tick = 100;
    app.update();

    let pop = app.world().entity(pop_entity);
    assert!(pop.contains::<ExistentialCrisis>(), "Pop should have crisis after failed audit");

    // Advance 500 ticks to clear duration
    for _ in 0..500 {
        app.update();
    }

    let pop = app.world().entity(pop_entity);
    assert!(!pop.contains::<ExistentialCrisis>(), "Crisis should decay after 500 updates");
}
