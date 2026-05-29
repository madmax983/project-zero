use bevy::prelude::*;
use scale::layer1::entities::pop::Pop;
use scale::layer1::psychology::memory::{Memories, MemoryType};
use scale::layer2::bombardment::BombardmentEvent;
use scale::layer3::diplomacy::flesh_tax::{
    process_flesh_tax_failure, process_flesh_tax_payment, FleshTaxFailedEvent, FleshTaxPaymentEvent,
};

#[test]
fn test_flesh_tax_integration_payment() {
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin);
    app.add_event::<FleshTaxPaymentEvent>();
    app.add_systems(Update, process_flesh_tax_payment);

    let pop1 = app.world_mut().spawn((Pop {}, Name::new("Sacrifice"))).id();
    let pop2 = app.world_mut().spawn((Pop {}, Name::new("Survivor"))).id();

    app.world_mut().send_event(FleshTaxPaymentEvent {
        tribute_pops: vec![pop1],
    });

    app.update();

    assert!(
        app.world().get_entity(pop1).is_err(),
        "Sacrificed pop should be despawned"
    );

    let memories = app.world().get::<Memories>(pop2);
    assert!(memories.is_some(), "Survivor should have memories");
    let has_trauma = memories
        .unwrap()
        .items
        .iter()
        .any(|m| m.memory_type == MemoryType::FleshTaxTrauma);
    assert!(has_trauma, "Survivor should have trauma memory");
}

#[test]
fn test_flesh_tax_integration_failure() {
    let mut app = App::new();
    app.add_event::<BombardmentEvent>();
    app.add_event::<FleshTaxFailedEvent>();
    app.add_systems(Update, process_flesh_tax_failure);

    app.world_mut().send_event(FleshTaxFailedEvent {});

    app.update();

    let bombardment_events = app.world().resource::<Events<BombardmentEvent>>();
    assert_eq!(
        bombardment_events.get_cursor().len(bombardment_events),
        1,
        "Failure should trigger an orbital bombardment"
    );
}
