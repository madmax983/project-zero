use bevy::prelude::*;
use scale::layer1::shipbreaking::{SpawnCrashedShipEvent, MineEvent, HullTile, Mineable, YieldsOnMine, Miner, spawn_crashed_ship_system, mine_system, hull_destroyed_system};
use scale::layer1::health::Health;
use scale::layer1::resources::{ResourceItem, ResourceType};
use scale::layer1::map::GridPosition;

#[test]
fn test_shipbreaking_integration_seam() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (spawn_crashed_ship_system, mine_system, hull_destroyed_system).chain());
    app.world_mut().init_resource::<Events<SpawnCrashedShipEvent>>();
    app.world_mut().init_resource::<Events<MineEvent>>();

    // Spawn a crashed ship
    app.world_mut().send_event(SpawnCrashedShipEvent { location: GridPosition { x: 10, y: 10 } });
    app.update();

    let mut hull_tiles = app.world_mut().query::<(Entity, &HullTile)>();
    let hull_entity = hull_tiles.iter(&app.world()).next().unwrap().0;

    let miner_entity = app.world_mut().spawn(Miner { tool_tier: 3 }).id();

    // Mine the hull
    app.world_mut().send_event(MineEvent { miner: miner_entity, target: hull_entity });

    // We need multiple updates to let health drain or we just fake a low health
    let mut health = app.world_mut().get_mut::<Health>(hull_entity).unwrap();
    health.current = 10.0; // make it break next mine

    app.update(); // The mine system will deal 10 damage and hull destroyed system will despawn it

    // Validate dropped resource
    let mut resources = app.world_mut().query::<&ResourceItem>();
    assert!(resources.iter(&app.world()).count() > 0, "Resource should be dropped");
}
