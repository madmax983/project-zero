use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::existential_audit_chronicle_bridge;
use scale::layer1::economy::existential_audit::{
    existential_audit_system, ExistentialAuditCompletedEvent, IndustrialBuilding, PrecursorAI,
};
use scale::layer1::pop::Pop;
use scale::shared::time::SimulationTime;

#[test]
fn test_existential_audit_failed_chronicle_bridge() {
    let mut app = App::new();

    // 1. Setup Resources
    app.init_resource::<SimulationTime>();
    app.insert_resource(PrecursorAI {
        next_audit_tick: 10,
    });

    // 2. Setup Events
    app.add_event::<ExistentialAuditCompletedEvent>();
    app.add_event::<AddChronicleEvent>();

    // 3. Register systems (Audit -> Bridge)
    app.add_systems(
        Update,
        (existential_audit_system, existential_audit_chronicle_bridge).chain(),
    );

    // 4. Setup World Entities
    // Spawn a pop
    app.world_mut().spawn((Pop,));
    // Spawn a highly industrial building to guarantee audit failure
    app.world_mut().spawn(IndustrialBuilding {
        efficiency: 100.0,
        cultural_value: 0.0,
    });

    // 5. Advance time to trigger audit
    app.world_mut().resource_mut::<SimulationTime>().tick = 10;

    // Act
    app.update();

    // Assert
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit exactly one chronicle event");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(
        events[0].text.contains("audit failed"),
        "Chronicle event text should indicate failure. Got: {}",
        events[0].text
    );
}

#[test]
fn test_existential_audit_passed_chronicle_bridge() {
    let mut app = App::new();

    // 1. Setup Resources
    app.init_resource::<SimulationTime>();
    app.insert_resource(PrecursorAI {
        next_audit_tick: 10,
    });

    // 2. Setup Events
    app.add_event::<ExistentialAuditCompletedEvent>();
    app.add_event::<AddChronicleEvent>();

    // 3. Register systems (Audit -> Bridge)
    app.add_systems(
        Update,
        (existential_audit_system, existential_audit_chronicle_bridge).chain(),
    );

    // 4. Setup World Entities
    // Spawn a pop
    app.world_mut().spawn((Pop,));
    // Spawn a highly cultural building to guarantee audit success
    app.world_mut().spawn(IndustrialBuilding {
        efficiency: 10.0,
        cultural_value: 100.0,
    });

    // 5. Advance time to trigger audit
    app.world_mut().resource_mut::<SimulationTime>().tick = 10;

    // Act
    app.update();

    // Assert
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit exactly one chronicle event");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(
        events[0].text.contains("audit concluded"),
        "Chronicle event text should indicate success. Got: {}",
        events[0].text
    );
}
