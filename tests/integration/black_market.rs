use bevy::MinimalPlugins;
use bevy_ecs::prelude::*;
use scale::layer1::admin::{calculate_admin_stats, AdminConsumer, AdminProvider, AdminStats};
use scale::layer1::black_market::{black_market_spawn_system, smuggler_trade_system, ColonyStats, Smuggler};
use scale::layer1::integration::update_unmet_luxury_system;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;

#[test]
fn integration_black_market_unmet_needs_spawns_smuggler() {
    let mut app = bevy_app::App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ColonyStats>();

    app.add_systems(bevy_app::Update, (
        update_unmet_luxury_system,
        black_market_spawn_system,
    ).chain());

    // Setup: 60 Pops with very low leisure (leisure < 0.2 means unmet luxury)
    for _ in 0..60 {
        app.world_mut().spawn((
            Pop,
            Needs { leisure: 0.1, ..Default::default() },
        ));
    }

    // Act
    app.update();

    // Assert: unmet luxury should be 60, triggering smuggler spawn (requires > 50)
    let stats = app.world().get_resource::<ColonyStats>().unwrap();
    assert_eq!(stats.unmet_luxury, 60);

    let smugglers = app.world_mut().query::<&Smuggler>().iter(app.world()).count();
    assert!(smugglers > 0, "Smuggler should have spawned");
}

#[test]
fn integration_black_market_corruption_reduces_efficiency() {
    let mut app = bevy_app::App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ColonyStats>();
    app.init_resource::<AdminStats>();

    app.add_systems(bevy_app::Update, (
        smuggler_trade_system,
        calculate_admin_stats,
    ).chain());

    // Setup: 10 supply, 10 demand -> normal efficiency 1.0
    app.world_mut().spawn(AdminProvider { amount: 10.0 });
    app.world_mut().spawn(AdminConsumer { demand: 10.0 });

    app.world_mut().spawn(Smuggler); // Spawn smuggler to trade

    // Act
    app.update();

    // Assert: smuggler traded, corruption increased, admin efficiency reduced
    let stats = app.world().get_resource::<ColonyStats>().unwrap();
    assert_eq!(stats.corruption, 1.0);

    let admin = app.world().get_resource::<AdminStats>().unwrap();
    // Assuming corruption reduces efficiency by some factor (e.g., 0.05 per corruption)
    assert!(admin.efficiency < 1.0, "Corruption should reduce efficiency");
}
