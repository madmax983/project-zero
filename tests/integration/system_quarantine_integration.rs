use bevy::prelude::*;
use scale::layer2::events_new::system_quarantine::{
    apply_quarantine_effects, handle_quarantine_decay, Colony, ResourceStockpile, SystemLocation,
    SystemQuarantine, TradeHub, UnrestLevel, WarlordFaction,
};

#[test]
fn test_system_quarantine_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(
        Update,
        (apply_quarantine_effects, handle_quarantine_decay).chain(),
    );

    let system_entity = app
        .world_mut()
        .spawn((SystemLocation, TradeHub { active: true }, SystemQuarantine))
        .id();

    let _colony_entity = app
        .world_mut()
        .spawn((
            Colony,
            SystemQuarantine,
            UnrestLevel(145),
            ResourceStockpile {
                uncontaminated_soil: 50,
            },
        ))
        .id();

    app.update();

    assert!(!app.world().get::<TradeHub>(system_entity).unwrap().active);

    let has_warlords = app
        .world_mut()
        .query::<&WarlordFaction>()
        .iter(app.world())
        .count()
        > 0;
    assert!(has_warlords);
}
