use bevy_ecs::prelude::*;
use scale::layer1::quirks::{PlanetaryTrait, PlanetaryTraits};
use scale::layer2::integration::escape_velocity_traits_bridge_system;
use scale::layer2::trade::escape_velocity::{process_launch_system, CargoItem, LaunchShipEvent, PlanetaryGravity, TradeManifest};
use scale::layer1::resources::ColonyResources;
use bevy_app::App;

#[test]
fn test_integration_escape_velocity_traits() {
    let mut app = App::new();

    // Init Resources
    app.insert_resource(PlanetaryGravity::default());
    app.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::HighGravity]));
    app.insert_resource(ColonyResources {
        fuel: 2000.0,
        ..Default::default()
    });
    app.add_event::<LaunchShipEvent>();

    // Add Systems
    app.add_systems(
        bevy_app::Update,
        (
            escape_velocity_traits_bridge_system,
            process_launch_system,
        ).chain()
    );

    let manifest_entity = app.world_mut().spawn(TradeManifest {
        items: vec![CargoItem { mass: 50.0, value: 500.0 }]
    }).id();

    app.world_mut().resource_mut::<Events<LaunchShipEvent>>().send(LaunchShipEvent {
        manifest_entity,
    });

    app.update();

    let post_res = app.world().resource::<ColonyResources>();
    // Launch cost with HighGravity (2.5g)
    // Cost = 100 + (50 * 2.5 * 10) = 1350
    // 2000 - 1350 = 650.0
    assert_eq!(post_res.fuel, 650.0);
    assert!(app.world().get_entity(manifest_entity).is_err(), "Entity should have despawned after launch");
}
