#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::economy::photophobic::{
        photophobic_degradation_system, update_photophobic_light_level_system, LightLevel,
        PhotophobicResource,
    };
    use scale::layer1::lighting::LightMap;
    use scale::layer1::map::GridPosition;

    #[test]
    fn test_photophobic_resource_light_level_updated_from_lightmap() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add LightMap resource
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0); // Bright light at (5, 5)
        app.insert_resource(light_map);

        // Register the systems
        app.add_systems(
            Update,
            (
                update_photophobic_light_level_system,
                photophobic_degradation_system,
            )
                .chain(),
        );

        // Spawn a PhotophobicResource at the lit position
        let resource_lit = app
            .world_mut()
            .spawn((
                PhotophobicResource {
                    amount: 100.0,
                    degradation_rate: 10.0,
                },
                LightLevel { intensity: 0.0 }, // Starts dark
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Spawn a PhotophobicResource at a dark position
        let resource_dark = app
            .world_mut()
            .spawn((
                PhotophobicResource {
                    amount: 100.0,
                    degradation_rate: 10.0,
                },
                LightLevel { intensity: 0.0 },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run update
        app.update();

        // Verify lit resource degraded
        let res_lit = app
            .world()
            .get::<PhotophobicResource>(resource_lit)
            .unwrap();
        assert!(
            res_lit.amount < 100.0,
            "Lit resource should have degraded, amount is {}",
            res_lit.amount
        );

        let light_level_lit = app.world().get::<LightLevel>(resource_lit).unwrap();
        assert_eq!(
            light_level_lit.intensity, 1.0,
            "LightLevel should be updated from LightMap"
        );

        // Verify dark resource did not degrade
        let res_dark = app
            .world()
            .get::<PhotophobicResource>(resource_dark)
            .unwrap();
        assert_eq!(res_dark.amount, 100.0, "Dark resource should not degrade");

        let light_level_dark = app.world().get::<LightLevel>(resource_dark).unwrap();
        assert_eq!(light_level_dark.intensity, 0.0, "LightLevel should be 0.0");
    }
}
