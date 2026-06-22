use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::integration::phantom_limb_chronicle_bridge;
use scale::layer2::trade::phantom_limb_logistics::AuditRiskEvent;
use scale::layer1::social::factions::FactionId;

#[test]
fn test_phantom_limb_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<AuditRiskEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, phantom_limb_chronicle_bridge);

    app.world_mut().send_event(AuditRiskEvent {
        target_faction: FactionId::MinersGuild,
        severity: 0.5,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut count = 0;
    for ev in reader.read(events) {
        assert_eq!(ev.importance, EventImportance::Major);
        assert!(ev.text.contains("Phantom Limb"));
        count += 1;
    }
    assert_eq!(count, 1, "Should emit one Chronicle event");
}
