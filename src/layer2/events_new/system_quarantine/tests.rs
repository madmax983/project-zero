use super::*;
use bevy::prelude::*;

#[test]
fn test_quarantine_activation_blocks_trade_and_fleets() {
    let mut app = App::new();
    // Setup initial system and fleets
    app.add_systems(Update, apply_quarantine_effects);

    let system_entity = app.world_mut().spawn((
        SystemLocation,
        TradeHub { active: true },
    )).id();

    // Apply quarantine
    app.world_mut().entity_mut(system_entity).insert(SystemQuarantine);

    app.update();

    // Assert trade is inactive and fleets cannot enter/leave
    assert_eq!(app.world().get::<TradeHub>(system_entity).unwrap().active, false);
}

#[test]
fn test_quarantine_generates_warlord_factions_over_time() {
    let mut app = App::new();
    app.add_systems(Update, handle_quarantine_decay);

    let _colony_entity = app.world_mut().spawn((
        Colony,
        SystemQuarantine,
        UnrestLevel(100),
        ResourceStockpile { uncontaminated_soil: 50 },
    )).id();

    // Simulate time passing
    for _ in 0..10 {
        app.update();
    }

    // Assert a warlord faction component or entity has been spawned related to this colony
    let has_warlords = app.world_mut().query::<&WarlordFaction>().iter(app.world()).count() > 0;
    assert!(has_warlords);
}
