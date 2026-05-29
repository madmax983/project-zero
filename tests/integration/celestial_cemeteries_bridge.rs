#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::culture::celestial_cemeteries::{
        calculate_launch_risk_system, LaunchSequence, OrbitalCemetery,
    };

    use scale::layer1::resources::ColonyResources;
    use scale::layer2::syzygy::PlanetaryGravity;
    use scale::layer2::trade::escape_velocity::{
        process_launch_system, CargoItem, LaunchShipEvent, TradeManifest,
    };

    // Test that calculating launch risk system properly hooks in with orbital cemetery.
    // And when the ship launches via TradeManifest and LaunchShipEvent, the launch cost is still based on CargoItem and PlanetaryGravity.
    // We should write a bridge test here since Celestial Cemeteries has no integration tests yet.

    #[test]
    fn test_celestial_cemeteries_launch_risk_bridge() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<OrbitalCemetery>()
            .insert_resource(PlanetaryGravity::default())
            .insert_resource(ColonyResources {
                fuel: 2000.0,
                ..Default::default()
            });

        app.add_event::<LaunchShipEvent>();

        app.add_systems(
            Update,
            (calculate_launch_risk_system, process_launch_system).chain(),
        );

        let mut cemetery = app.world_mut().resource_mut::<OrbitalCemetery>();
        cemetery.coffin_count = 500; // 5% extra risk

        let manifest_entity = app
            .world_mut()
            .spawn((
                TradeManifest {
                    items: vec![CargoItem {
                        mass: 50.0,
                        value: 500.0,
                    }],
                },
                LaunchSequence { base_risk: 0.05 },
            ))
            .id();

        app.update();

        let risk = app
            .world()
            .get::<LaunchSequence>(manifest_entity)
            .unwrap()
            .base_risk;
        assert!(
            risk > 0.05,
            "Launch risk should be increased due to celestial cemetery coffins"
        );

        app.world_mut()
            .resource_mut::<Events<LaunchShipEvent>>()
            .send(LaunchShipEvent { manifest_entity });

        app.update();

        // Ship should be launched and despawned because fuel cost is 100 + 50*1*10 = 600, fuel was 2000.
        assert!(app.world().get_entity(manifest_entity).is_err());
        let post_res = app.world().resource::<ColonyResources>();
        assert_eq!(post_res.fuel, 1400.0);
    }
}
