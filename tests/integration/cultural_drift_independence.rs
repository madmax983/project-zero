use bevy::prelude::*;
use scale::layer2::culture::cultural_drift::{
    handle_independence_system, ColonyMarker, CulturalDrift, Faction,
};
use scale::layer2::integration::cultural_drift_independence_bridge;
use scale::layer3::diplomacy::system_sovereignty::{
    ColonyStatus, DeclarationOfIndependenceEvent, FactionRelations, WarDeclarationEvent,
};

#[test]
fn test_cultural_drift_triggers_independence_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<DeclarationOfIndependenceEvent>();
    app.add_event::<WarDeclarationEvent>();

    app.insert_resource(ColonyStatus {
        is_sovereign: false,
        overlord_id: Some(1),
    });
    app.insert_resource(FactionRelations::default());

    app.add_systems(
        Update,
        (
            handle_independence_system,
            cultural_drift_independence_bridge,
        )
            .chain(),
    );

    let colony_id = app
        .world_mut()
        .spawn((
            ColonyMarker,
            CulturalDrift {
                value: 101.0,
                independence_threshold: 100.0,
            },
            Faction { id: 0 },
        ))
        .id();

    app.update();

    let events = app
        .world()
        .resource::<Events<DeclarationOfIndependenceEvent>>();
    let reader = events.get_cursor();
    assert_eq!(
        reader.len(events),
        1,
        "A DeclarationOfIndependenceEvent should be emitted when drift reaches threshold"
    );

    let faction = app.world().get::<Faction>(colony_id).unwrap();
    assert_ne!(faction.id, 0, "Faction ID should be changed");
}
